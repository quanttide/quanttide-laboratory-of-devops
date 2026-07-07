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
	gh       *GitHubClient
	store    *ShelvedStore
	logger   *slog.Logger
	scopes   []Scope
	mu       sync.RWMutex
	lastReport *ConvergeReport
}

func Routes(h *handler) http.Handler {
	r := chi.NewRouter()
	r.Get("/health", h.Health)
	r.Get("/scan", h.ScanAll)
	r.Get("/scan/{scope}", h.Scan)
	r.Post("/repair/{scope}", h.Repair)
	r.Get("/report", h.Report)
	return r
}

func NewHandler(gh *GitHubClient, store *ShelvedStore, logger *slog.Logger, scopes []Scope) *handler {
	return &handler{gh: gh, store: store, logger: logger, scopes: scopes}
}

func (h *handler) Health(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
}

func (h *handler) Scan(w http.ResponseWriter, r *http.Request) {
	scope := Scope(chi.URLParam(r, "scope"))
	if scope.IsZero() {
		http.Error(w, `{"error":"missing scope"}`, http.StatusBadRequest)
		return
	}

	scanner := NewScanner(h.gh)
	result, err := scanner.ScanScope(r.Context(), scope)
	if err != nil {
		h.logger.Error("scan failed", "scope", scope, "error", err)
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
		Errors:       stats.Abnormal - stats.CausalBreaks,
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
	scope := Scope(chi.URLParam(r, "scope"))
	if scope.IsZero() {
		http.Error(w, `{"error":"missing scope"}`, http.StatusBadRequest)
		return
	}

	scanner := NewScanner(h.gh)
	result, err := scanner.ScanScope(r.Context(), scope)
	if err != nil {
		h.logger.Error("pre-repair scan failed", "scope", scope, "error", err)
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
		h.logger.Error("repair failed", "scope", scope, "error", err)
		http.Error(w, `{"error":"`+err.Error()+`"}`, http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(action)
}

func (h *handler) converge(ctx context.Context, scanner *Scanner) []ScanResult {
	var results []ScanResult
	for _, scope := range h.scopes {
		select {
		case <-ctx.Done():
			return results
		default:
		}
		result, err := scanner.ScanScope(ctx, scope)
		if err != nil {
			h.logger.Error("converge scan", "scope", scope, "error", err)
			results = append(results, ScanResult{Scope: scope, Status: StatusUnreleased, Summary: err.Error()})
			continue
		}
		results = append(results, *result)
	}
	return results
}

func reportTime() time.Time {
	return time.Now().UTC()
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

		rescan, err := scanner.ScanScope(ctx, r.Scope)
		if err == nil {
			results[i] = *rescan
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
		Errors:       stats.Abnormal - stats.CausalBreaks,
	}

	h.mu.Lock()
	h.lastReport = &report
	h.mu.Unlock()

	return &report
}
