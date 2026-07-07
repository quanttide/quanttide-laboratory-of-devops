package main

import (
	"context"
	"encoding/json"
	"log/slog"
	"net/http"
	"sync"
	"time"

	"github.com/go-chi/chi/v5"
)

type handler struct {
	gh         *GitHubClient
	store      *ShelvedStore
	logger     *slog.Logger
	scopes     []ScopeMapping
	mu         sync.RWMutex
	lastReport *ConvergeReport
}

func Routes(h *handler) http.Handler {
	r := chi.NewRouter()
	r.Get("/health", h.Health)
	r.Get("/scan", h.ScanAll)
	r.Get("/scan/{scope:*}", h.Scan)
	r.Post("/repair/{scope:*}", h.Repair)
	r.Get("/report", h.Report)
	return r
}

func NewHandler(gh *GitHubClient, store *ShelvedStore, logger *slog.Logger, scopes []ScopeMapping) *handler {
	return &handler{gh: gh, store: store, logger: logger, scopes: scopes}
}

func (h *handler) Health(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
}

func (h *handler) resolveScope(scopeStr string) (*ScopeMapping, error) {
	s := Scope(scopeStr)
	m, err := ParseScope(s)
	if err != nil {
		return nil, err
	}
	// Try to find dir from pre-resolved scopes
	for _, sm := range h.scopes {
		if sm.Owner == m.Owner && sm.Repo == m.Repo && sm.Name == m.Name {
			return &sm, nil
		}
	}
	return &m, nil
}

func (h *handler) Scan(w http.ResponseWriter, r *http.Request) {
	scopeStr := chi.URLParam(r, "*")
	if scopeStr == "" {
		http.Error(w, `{"error":"missing scope"}`, http.StatusBadRequest)
		return
	}

	m, err := h.resolveScope(scopeStr)
	if err != nil {
		http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusBadRequest)
		return
	}

	scanner := NewScanner(h.gh)
	result, err := scanner.ScanScope(r.Context(), *m)
	if err != nil {
		h.logger.Error("scan failed", "scope", scopeStr, "error", err)
		http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(result)
}

func (h *handler) ScanAll(w http.ResponseWriter, r *http.Request) {
	scanner := NewScanner(h.gh)
	results := h.converge(r.Context(), scanner)
	stats := Aggregate(results)

	report := ConvergeReport{
		Timestamp:    reportTime(),
		Total:        stats.Total,
		Normal:       stats.Normal,
		Shelved:      stats.Shelved,
		CausalBreaks: stats.CausalBreaks,
		PendingRel:   stats.PendingRel,
		Errors:       stats.Abnormal - stats.CausalBreaks - stats.PendingRel,
		Results:      results,
	}

	h.mu.Lock()
	h.lastReport = &report
	h.mu.Unlock()

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(report)
}

func (h *handler) Report(w http.ResponseWriter, r *http.Request) {
	h.mu.RLock()
	report := h.lastReport
	h.mu.RUnlock()

	if report == nil {
		http.Error(w, `{"error":"no report yet, run GET /scan first"}`, http.StatusNotFound)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(report)
}

func (h *handler) Repair(w http.ResponseWriter, r *http.Request) {
	scopeStr := chi.URLParam(r, "*")
	if scopeStr == "" {
		http.Error(w, `{"error":"missing scope"}`, http.StatusBadRequest)
		return
	}

	m, err := h.resolveScope(scopeStr)
	if err != nil {
		http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusBadRequest)
		return
	}

	scanner := NewScanner(h.gh)
	result, err := scanner.ScanScope(r.Context(), *m)
	if err != nil {
		h.logger.Error("pre-repair scan failed", "scope", scopeStr, "error", err)
		http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
		return
	}

	if !result.Repairable {
		http.Error(w, `{"error":"scope is not repairable"}`, http.StatusConflict)
		return
	}

	repairer := NewRepairer(h.gh, h.store, h.logger)
	action, err := repairer.Repair(r.Context(), *result)
	if err != nil {
		h.logger.Error("repair failed", "scope", scopeStr, "error", err)
		http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(action)
}

func (h *handler) converge(ctx context.Context, scanner *Scanner) []ScanResult {
	var results []ScanResult
	for _, m := range h.scopes {
		select {
		case <-ctx.Done():
			return results
		default:
		}
		result, err := scanner.ScanScope(ctx, m)
		if err != nil {
			h.logger.Error("converge scan", "scope", scopeStr(m), "error", err)
			results = append(results, ScanResult{Scope: Scope(m.Owner + "/" + m.Repo), Status: StatusUnreleased, Summary: err.Error()})
			continue
		}
		results = append(results, *result)
	}
	return results
}

func (h *handler) convergeAndRepair(ctx context.Context) *ConvergeReport {
	scanner := NewScanner(h.gh)
	results := h.converge(ctx, scanner)
	var fixed int

	for i, r := range results {
		if !r.Repairable {
			continue
		}
		repairer := NewRepairer(h.gh, h.store, h.logger)
		_, err := repairer.Repair(ctx, r)
		if err != nil {
			h.logger.Error("converge repair", "scope", r.Scope, "error", err)
			continue
		}
		fixed++

		// Re-scan after repair
		m, _ := h.resolveScope(string(r.Scope))
		if m != nil {
			rescan, err := scanner.ScanScope(ctx, *m)
			if err == nil {
				results[i] = *rescan
			}
		}
	}

	stats := Aggregate(results)
	report := ConvergeReport{
		Timestamp:    reportTime(),
		Total:        stats.Total,
		Normal:       stats.Normal,
		Fixed:        fixed,
		Shelved:      stats.Shelved,
		CausalBreaks: stats.CausalBreaks,
		PendingRel:   stats.PendingRel,
		Errors:       stats.Abnormal - stats.CausalBreaks - stats.PendingRel,
	}

	h.mu.Lock()
	h.lastReport = &report
	h.mu.Unlock()

	return &report
}

func scopeStr(m ScopeMapping) string {
	if m.Name == "" {
		return m.Owner + "/" + m.Repo
	}
	return m.Owner + "/" + m.Repo + "/" + m.Name
}

func reportTime() time.Time {
	return time.Now().UTC()
}
