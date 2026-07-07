package main

import (
	"fmt"
	"strings"
)

// ScopeMapping maps a scope name to its directory within the repo.
// Matches CLI convention: scope name is tag prefix, dir is CHANGELOG location.
type ScopeMapping struct {
	Owner string
	Repo  string
	Name  string // scope name, e.g. "cli", "rust". empty = root
	Dir   string // scope dir within repo, fallback = Name
}

// ParseScope parses "owner/repo[/scopeName]" into ScopeMapping.
// Dir defaults to Name (CLI fallback when no contract.yaml).
func ParseScope(s Scope) (ScopeMapping, error) {
	parts := strings.SplitN(string(s), "/", 3)
	if len(parts) < 2 || parts[0] == "" || parts[1] == "" {
		return ScopeMapping{}, fmt.Errorf("invalid scope %q: expected owner/repo[/scope]", s)
	}
	m := ScopeMapping{Owner: parts[0], Repo: parts[1]}
	if len(parts) == 3 && parts[2] != "" {
		m.Name = parts[2]
		m.Dir = parts[2] // CLI fallback: dir = scope name
	}
	return m, nil
}

// ChangelogPath returns the CHANGELOG path for this scope.
// Root reads CHANGELOG.md; scoped reads {Dir}/CHANGELOG.md.
func (m ScopeMapping) ChangelogPath() string {
	if m.Name == "" {
		return "CHANGELOG.md"
	}
	return m.Dir + "/CHANGELOG.md"
}

// splitTag splits a tag like "cli/v0.10.0" into ("cli", "v0.10.0").
// Root tags like "v0.1.0" return ("", "v0.1.0").
// Matches CLI's parse_tag: tag.split_once('/').
func splitTag(tag string) (scope, version string) {
	i := strings.IndexByte(tag, '/')
	if i < 0 {
		return "", tag
	}
	return tag[:i], tag[i+1:]
}

func (s Scope) IsZero() bool {
	return s == ""
}
