package main

import (
	"context"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
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

	scopes := discoverScopes()

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

func discoverScopes() []Scope {
	return []Scope{
		"quanttide/qtcloud-devops",
		"quanttide/quanttide-devops-toolkit",
		"quanttide/qtcloud-code",
		"quanttide/qtadmin",
		"quanttide/quanttide-website",
	}
}

type authTransport struct {
	token string
}

func (t *authTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	req = req.Clone(req.Context())
	req.Header.Set("Authorization", "Bearer "+t.token)
	return http.DefaultTransport.RoundTrip(req)
}
