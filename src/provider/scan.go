package main

import (
	"context"
	"fmt"
	"strings"
)

type Scanner struct {
	gh *GitHubClient
}

func NewScanner(gh *GitHubClient) *Scanner {
	return &Scanner{gh: gh}
}

func (s *Scanner) ScanScope(ctx context.Context, scope Scope) (*ScanResult, error) {
	m, err := ParseScope(scope)
	if err != nil {
		return nil, err
	}

	state, _, err := s.scanArtifacts(ctx, m.Owner, m.Repo)
	if err != nil {
		return nil, fmt.Errorf("scan %s: %w", scope, err)
	}
	state.Scope = scope

	judge := Judge(*state)
	return &ScanResult{
		Scope:      scope,
		State:      *state,
		Status:     judge.Status,
		Summary:    judge.Summary,
		Repairable: judge.Repairable,
	}, nil
}

func (s *Scanner) scanArtifacts(ctx context.Context, owner, repo string) (*ArtifactState, string, error) {
	state := &ArtifactState{Owner: owner, Repo: repo}

	tags, err := s.gh.ListTags(ctx, owner, repo)
	if err != nil {
		return nil, "", err
	}
	if len(tags) > 0 {
		state.HasTag = true
		state.Version = strings.TrimPrefix(tags[0].GetRef(), "refs/tags/")
	}

	changelog, err := s.gh.GetChangelog(ctx, owner, repo)
	state.HasChangelog = err == nil && strings.Contains(changelog, state.Version)

	releases, err := s.gh.ListReleases(ctx, owner, repo)
	if err == nil {
		for _, rel := range releases {
			if rel.GetTagName() == state.Version {
				state.HasRelease = true
				break
			}
		}
	}

	return state, state.Version, nil
}
