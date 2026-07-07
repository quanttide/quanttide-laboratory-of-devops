package main

import (
	"encoding/json"
	"log/slog"
	"net/http"

	"github.com/go-chi/chi/v5"
)

type handler struct {
	gh       *GitHubClient
	store    *ShelvedStore
	logger   *slog.Logger
}

func Routes(h *handler) http.Handler {
	r := chi.NewRouter()
	r.Get("/health", h.Health)
	r.Get("/scan/{scope}", h.Scan)
	r.Post("/repair/{scope}", h.Repair)
	return r
}

func NewHandler(gh *GitHubClient, store *ShelvedStore, logger *slog.Logger) *handler {
	return &handler{gh: gh, store: store, logger: logger}
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
