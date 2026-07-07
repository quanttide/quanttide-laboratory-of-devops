package main

import (
	"context"
	"fmt"
	"log/slog"
)

type Repairer struct {
	gh     *GitHubClient
	store  *ShelvedStore
	logger *slog.Logger
}

func NewRepairer(gh *GitHubClient, store *ShelvedStore, logger *slog.Logger) *Repairer {
	return &Repairer{gh: gh, store: store, logger: logger}
}

func (r *Repairer) Repair(ctx context.Context, result ScanResult) (*RepairAction, error) {
	m, err := ParseScope(result.Scope)
	if err != nil {
		return nil, err
	}

	switch result.Status {
	case StatusMissingCL:
		r.logger.Info("repairing missing CHANGELOG", "scope", result.Scope)
		return r.repairChangelog(ctx, m.Owner, m.Repo, fullTag(m.Name, result.State.TagVersion))

	case StatusMissingRel:
		r.logger.Info("repairing missing Release", "scope", result.Scope)
		return r.repairRelease(ctx, m.Owner, m.Repo, fullTag(m.Name, result.State.TagVersion))

	case StatusOnlyTag:
		r.logger.Info("shelving scope", "scope", result.Scope)
		item := ShelvedItem{
			Scope:   result.Scope,
			Version: fullTag(m.Name, result.State.TagVersion),
			Reason:  "只有 tag，缺 CHANGELOG 和 Release，无法自动修复",
		}
		if err := r.store.Append(item); err != nil {
			return nil, fmt.Errorf("shelve %s: %w", result.Scope, err)
		}
		return &RepairAction{Scope: result.Scope, Type: "shelved"}, nil

	case StatusPendingRel:
		version := fullTag(m.Name, result.State.ChangelogVersion)
		r.logger.Info("pending release — creating tag+release from CL", "scope", result.Scope, "version", version)
		return r.repairRelease(ctx, m.Owner, m.Repo, version)

	default:
		return nil, fmt.Errorf("scope %s is not repairable (status: %s)", result.Scope, result.Status)
	}
}

func (r *Repairer) repairChangelog(ctx context.Context, owner, repo, version string) (*RepairAction, error) {
	title := fmt.Sprintf("docs: 添加 %s CHANGELOG 条目", version)
	body := fmt.Sprintf("自动生成 %s 的 CHANGELOG 条目", version)
	if err := r.gh.CreatePR(ctx, owner, repo, title, body, "main", "main"); err != nil {
		return nil, err
	}
	return &RepairAction{Scope: Scope(owner + "/" + repo), Type: "changelog_pr"}, nil
}

func (r *Repairer) repairRelease(ctx context.Context, owner, repo, version string) (*RepairAction, error) {
	if err := r.gh.CreateRelease(ctx, owner, repo, version, ""); err != nil {
		return nil, err
	}
	return &RepairAction{Scope: Scope(owner + "/" + repo), Type: "release_created"}, nil
}

// fullTag prepends scope prefix to version if scope is not root.
// e.g. ("cli", "v0.2.1") → "cli/v0.2.1"
func fullTag(scopeName, version string) string {
	if scopeName == "" {
		return version
	}
	return scopeName + "/" + version
}
