package main

import (
	"context"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"
)

func main() {
	logger := slog.New(slog.NewJSONHandler(os.Stdout, nil))

	store := NewShelvedStore("shelved.json")

	token := os.Getenv("GITHUB_TOKEN")
	if token == "" {
		logger.Error("GITHUB_TOKEN is required")
		os.Exit(1)
	}
	gh, err := NewGitHubClientWithTransport(&authTransport{token: token})
	if err != nil {
		logger.Error("failed to create github client", "error", err)
		os.Exit(1)
	}

	scopes := discoverScopes(gh)

	h := NewHandler(gh, store, logger, scopes)
r := Routes(h)

	srv := &http.Server{
		Addr:    ":8080",
		Handler: r,
	}

	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	go func() {
		logger.Info("starting provider", "addr", srv.Addr)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			logger.Error("server error", "error", err)
			os.Exit(1)
		}
	}()

	interval := 5 * time.Minute
	if v := os.Getenv("CONVERGE_INTERVAL"); v != "" {
		if d, err := time.ParseDuration(v); err == nil {
			interval = d
		}
	}
	logger.Info("convergence loop", "interval", interval)

	go func() {
		runConverge(ctx, h, logger, interval)
	}()

	<-ctx.Done()
	logger.Info("shutting down")

	shutdownCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	if err := srv.Shutdown(shutdownCtx); err != nil {
		logger.Error("shutdown error", "error", err)
		os.Exit(1)
	}
	logger.Info("server stopped")
}

func runConverge(ctx context.Context, h *handler, logger *slog.Logger, interval time.Duration) {
	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			logger.Info("convergence cycle starting")
			report := h.convergeAndRepair(ctx)
			logger.Info("convergence cycle done",
				"total", report.Total,
				"normal", report.Normal,
				"fixed", report.Fixed,
				"shelved", report.Shelved,
				"causal_breaks", report.CausalBreaks,
			)
		}
	}
}

func discoverScopes(gh *GitHubClient) []ScopeMapping {
	repos := []struct{ owner, repo string }{
		{"quanttide", "qtcloud-devops"},
		{"quanttide", "quanttide-devops-toolkit"},
		{"quanttide", "qtcloud-code"},
		{"quanttide", "qtadmin"},
	}
	ctx := context.Background()
	var scopes []ScopeMapping

	for _, r := range repos {
		tags, err := gh.ListTags(ctx, r.owner, r.repo)
		if err != nil {
			continue
		}

		// 1. Discover scope names from tags (CLI: collect_tags_with_scope)
		scopeNames := map[string]bool{}
		for _, ref := range tags {
			tag := strings.TrimPrefix(ref.GetRef(), "refs/tags/")
			scopeName, _ := splitTag(tag)
			if scopeName != "" {
				scopeNames[scopeName] = true
			}
		}

		// 2. Auto-detect scope→dir mapping by scanning src/, packages/, apps/ (CLI: auto_detect)
		scopeDir := map[string]string{}
		for _, base := range []string{"src", "packages", "apps"} {
			subdirs, err := gh.ListDir(ctx, r.owner, r.repo, base)
			if err != nil {
				continue
			}
			for _, name := range subdirs {
				if scopeNames[name] {
					scopeDir[name] = base + "/" + name
				}
			}
		}

		// 3. Always add root scope
		scopes = append(scopes, ScopeMapping{Owner: r.owner, Repo: r.repo})

		// 4. Add scoped scopes with resolved dirs
		for name := range scopeNames {
			dir, ok := scopeDir[name]
			if !ok {
				dir = name // CLI fallback: scope name as dir
			}
			scopes = append(scopes, ScopeMapping{
				Owner: r.owner,
				Repo:  r.repo,
				Name:  name,
				Dir:   dir,
			})
		}
	}

	return scopes
}

type authTransport struct {
	token string
}

func (t *authTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	req = req.Clone(req.Context())
	req.Header.Set("Authorization", "Bearer "+t.token)
	return http.DefaultTransport.RoundTrip(req)
}
