package main

import (
	"context"
	"fmt"
	"regexp"
	"strings"
)

var versionRE = regexp.MustCompile(`v?\d+\.\d+\.\d+`)

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

	state, err := s.scanArtifacts(ctx, m.Owner, m.Repo)
	if err != nil {
		return nil, fmt.Errorf("scan %s: %w", scope, err)
	}
	state.Scope = scope
	state.TagCLMatch = state.HasTag && state.HasChangelog && matchVersion(state.TagVersion, state.ChangelogVersion)
	state.TagRelMatch = state.HasTag && state.HasRelease && matchVersion(state.TagVersion, state.ReleaseVersion)

	judge := Judge(*state)
	return &ScanResult{
		Scope:      scope,
		State:      *state,
		Status:     judge.Status,
		Summary:    judge.Summary,
		Repairable: judge.Repairable,
	}, nil
}

func (s *Scanner) scanArtifacts(ctx context.Context, owner, repo string) (*ArtifactState, error) {
	state := &ArtifactState{Owner: owner, Repo: repo}

	tags, err := s.gh.ListTags(ctx, owner, repo)
	if err != nil {
		return nil, err
	}
	if len(tags) > 0 {
		state.HasTag = true
		state.TagVersion = extractVersion(tags[0].GetRef())
	}

	changelog, err := s.gh.GetChangelog(ctx, owner, repo)
	if err == nil && changelog != "" {
		state.HasChangelog = true
		state.ChangelogVersion = extractFirstVersion(changelog)
	}

	releases, err := s.gh.ListReleases(ctx, owner, repo)
	if err == nil && len(releases) > 0 {
		state.HasRelease = true
		state.ReleaseVersion = extractVersion(releases[0].GetTagName())
	}

	return state, nil
}

func extractVersion(ref string) string {
	v := versionRE.FindString(ref)
	if v != "" && v[0] != 'v' {
		return "v" + v
	}
	return v
}

func extractFirstVersion(content string) string {
	v := versionRE.FindString(content)
	if v != "" && v[0] != 'v' {
		return "v" + v
	}
	return v
}

func matchVersion(a, b string) bool {
	if a == "" || b == "" {
		return false
	}
	return strings.TrimPrefix(a, "v") == strings.TrimPrefix(b, "v")
}
