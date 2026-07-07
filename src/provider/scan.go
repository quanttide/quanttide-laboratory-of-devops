package main

import (
	"context"
	"fmt"
	"regexp"
	"strconv"
	"strings"

	"github.com/google/go-github/v88/github"
)

var (
	versionRE    = regexp.MustCompile(`v?\d+\.\d+\.\d+`)
	changelogVer = regexp.MustCompile(`\[\s*v?(\d+\.\d+\.\d+)\s*\]`)
)

type Scanner struct {
	gh *GitHubClient
}

func NewScanner(gh *GitHubClient) *Scanner {
	return &Scanner{gh: gh}
}

func (s *Scanner) ScanScope(ctx context.Context, m ScopeMapping) (*ScanResult, error) {
	state, err := s.scanArtifacts(ctx, m)
	if err != nil {
		return nil, fmt.Errorf("scan %s: %w", scopeStr(m), err)
	}
	state.Scope = Scope(scopeStr(m))
	state.TagCLCmp = cmpTagCL(state.TagVersion, state.ChangelogVersion)
	state.TagRelMatch = state.HasTag && state.HasRelease && matchVersion(state.TagVersion, state.ReleaseVersion)

	judge := Judge(*state)
		return &ScanResult{
			Scope:      state.Scope,
		State:      *state,
		Status:     judge.Status,
		Summary:    judge.Summary,
		Repairable: judge.Repairable,
	}, nil
}

func (s *Scanner) scanArtifacts(ctx context.Context, m ScopeMapping) (*ArtifactState, error) {
	state := &ArtifactState{Owner: m.Owner, Repo: m.Repo}

	tags, err := s.gh.ListTags(ctx, m.Owner, m.Repo)
	if err != nil {
		return nil, err
	}
	if vers := filterAndExtract(tags, m.Name); len(vers) > 0 {
		state.HasTag = true
		state.TagVersion = latestSemver(vers)
	}

	clPath := m.ChangelogPath()
	changelog, err := s.gh.GetFile(ctx, m.Owner, m.Repo, clPath)
	if err == nil && changelog != "" {
		state.HasChangelog = true
		state.ChangelogVersion = extractFirstVersion(changelog)
	}

	releases, err := s.gh.ListReleases(ctx, m.Owner, m.Repo)
	if err == nil {
		if vers := extractReleaseVersions(releases, m.Name); len(vers) > 0 {
			state.HasRelease = true
			state.ReleaseVersion = latestSemver(vers)
		}
	}

	return state, nil
}

// filterAndExtract filters refs by scope name and extracts semver versions.
// Matches CLI's collect_tags_with_scope: split on first '/' to get scope name.
// Root scope (component="") matches tags without '/'.
// Scoped (component="cli") matches tags starting with "cli/".
func filterAndExtract(refs []*github.Reference, component string) []string {
	var vs []string
	for _, ref := range refs {
		tag := strings.TrimPrefix(ref.GetRef(), "refs/tags/")
		scopeName, verPart := splitTag(tag)
		if component == "" {
			if scopeName != "" {
				continue
			}
		} else if scopeName != component {
			continue
		}
		if v := extractVersion(verPart); v != "" {
			vs = append(vs, v)
		}
	}
	return vs
}

func extractReleaseVersions(rels []*github.RepositoryRelease, component string) []string {
	var vs []string
	for _, r := range rels {
		tag := r.GetTagName()
		scopeName, verPart := splitTag(tag)
		if component == "" {
			if scopeName != "" {
				continue
			}
		} else if scopeName != component {
			continue
		}
		if v := extractVersion(verPart); v != "" {
			vs = append(vs, v)
		}
	}
	return vs
}

func extractVersion(ref string) string {
	v := versionRE.FindString(ref)
	if v != "" && v[0] != 'v' {
		return "v" + v
	}
	return v
}

func extractFirstVersion(content string) string {
	m := changelogVer.FindStringSubmatch(content)
	if len(m) >= 2 {
		v := m[1]
		if v[0] != 'v' {
			return "v" + v
		}
		return v
	}
	return ""
}

func matchVersion(a, b string) bool {
	if a == "" || b == "" {
		return false
	}
	return strings.TrimPrefix(a, "v") == strings.TrimPrefix(b, "v")
}

// cmpTagCL compares tag and CL versions. Returns CmpLess (tag < CL → pending release),
// CmpEqual, or CmpGreater (tag > CL → CL behind / causal_break).
func cmpTagCL(tag, cl string) CmpResult {
	if tag == "" || cl == "" {
		return CmpEqual // can't compare
	}
	r := compareSemver(tag, cl)
	if r < 0 {
		return CmpLess
	} else if r > 0 {
		return CmpGreater
	}
	return CmpEqual
}

func latestSemver(versions []string) string {
	if len(versions) == 0 {
		return ""
	}
	best := versions[0]
	for _, v := range versions[1:] {
		if compareSemver(v, best) > 0 {
			best = v
		}
	}
	return best
}

func compareSemver(a, b string) int {
	a = strings.TrimPrefix(a, "v")
	b = strings.TrimPrefix(b, "v")
	pa := strings.SplitN(a, ".", 3)
	pb := strings.SplitN(b, ".", 3)
	for i := 0; i < 3; i++ {
		var va, vb int
		if i < len(pa) {
			va, _ = strconv.Atoi(pa[i])
		}
		if i < len(pb) {
			vb, _ = strconv.Atoi(pb[i])
		}
		if va != vb {
			return va - vb
		}
	}
	return 0
}
