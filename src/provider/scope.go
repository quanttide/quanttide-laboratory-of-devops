package main

import (
	"fmt"
	"strings"
)

type ScopeMapping struct {
	Owner string
	Repo  string
}

func ParseScope(s Scope) (ScopeMapping, error) {
	parts := strings.SplitN(string(s), "/", 2)
	if len(parts) != 2 || parts[0] == "" || parts[1] == "" {
		return ScopeMapping{}, fmt.Errorf("invalid scope %q: expected owner/repo", s)
	}
	return ScopeMapping{Owner: parts[0], Repo: parts[1]}, nil
}

func (s Scope) IsZero() bool {
	return s == ""
}
