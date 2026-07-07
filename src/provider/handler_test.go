package main

import (
	"encoding/json"
	"io"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"os"
	"testing"
)

func TestHealth(t *testing.T) {
	gh := &GitHubClient{}
	store := NewShelvedStore(os.TempDir() + "/test_shelved.json")
	defer os.Remove(os.TempDir() + "/test_shelved.json")
	logger := slog.New(slog.NewJSONHandler(io.Discard, nil))
	h := NewHandler(gh, store, logger)

	req := httptest.NewRequest("GET", "/health", nil)
	w := httptest.NewRecorder()
	h.Health(w, req)

	resp := w.Result()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}

	var body map[string]string
	json.NewDecoder(resp.Body).Decode(&body)
	if body["status"] != "ok" {
		t.Fatalf("expected status ok, got %v", body)
	}
}

func TestScopeParser(t *testing.T) {
	tests := []struct {
		input   Scope
		owner   string
		repo    string
		wantErr bool
	}{
		{Scope("owner/repo"), "owner", "repo", false},
		{Scope(""), "", "", true},
		{Scope("norepo"), "", "", true},
		{Scope("a/b/c"), "a", "b/c", false},
	}

	for _, tt := range tests {
		m, err := ParseScope(tt.input)
		if (err != nil) != tt.wantErr {
			t.Errorf("ParseScope(%q) error = %v, wantErr = %v", tt.input, err, tt.wantErr)
		}
		if err == nil {
			if m.Owner != tt.owner || m.Repo != tt.repo {
				t.Errorf("ParseScope(%q) = %+v, want owner=%q repo=%q", tt.input, m, tt.owner, tt.repo)
			}
		}
	}
}

func TestJudge(t *testing.T) {
	tests := []struct {
		state ArtifactState
		want  Status
	}{
		{ArtifactState{HasTag: true, HasChangelog: true, HasRelease: true}, StatusNormal},
		{ArtifactState{HasTag: true, HasChangelog: false, HasRelease: true}, StatusMissingCL},
		{ArtifactState{HasTag: true, HasChangelog: true, HasRelease: false}, StatusMissingRel},
		{ArtifactState{HasTag: true, HasChangelog: false, HasRelease: false}, StatusOnlyTag},
		{ArtifactState{HasTag: false, HasChangelog: false, HasRelease: false}, StatusUnreleased},
		{ArtifactState{HasTag: false, HasChangelog: true, HasRelease: true}, StatusUnreleased},
		{ArtifactState{HasTag: false, HasChangelog: true, HasRelease: false}, StatusUnreleased},
		{ArtifactState{HasTag: false, HasChangelog: false, HasRelease: true}, StatusUnreleased},
	}

	for _, tt := range tests {
		result := Judge(tt.state)
		if result.Status != tt.want {
			t.Errorf("Judge(%+v) = %s, want %s", tt.state, result.Status, tt.want)
		}
	}
}

func TestAggregate(t *testing.T) {
	results := []ScanResult{
		{Status: StatusNormal},
		{Status: StatusMissingCL},
		{Status: StatusMissingRel},
		{Status: StatusOnlyTag},
		{Status: StatusUnreleased},
	}
	stats := Aggregate(results)
	if stats.Total != 5 || stats.Normal != 1 || stats.Abnormal != 3 || stats.Shelved != 1 {
		t.Errorf("Aggregate = %+v", stats)
	}
}
