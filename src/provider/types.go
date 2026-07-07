package main

import "time"

type Scope string

type ArtifactState struct {
	Scope           Scope  `json:"scope"`
	Owner           string `json:"owner"`
	Repo            string `json:"repo"`
	TagVersion      string `json:"tag_version"`
	ChangelogVersion string `json:"changelog_version"`
	ReleaseVersion  string `json:"release_version"`
	HasTag          bool   `json:"has_tag"`
	HasChangelog    bool   `json:"has_changelog"`
	HasRelease      bool   `json:"has_release"`
	TagCLMatch      bool   `json:"tag_cl_match"`
	TagRelMatch     bool   `json:"tag_rel_match"`
}

type ScanResult struct {
	Scope      Scope        `json:"scope"`
	State      ArtifactState `json:"state"`
	Status     Status       `json:"status"`
	Summary    string       `json:"summary"`
	Repairable bool         `json:"repairable"`
}

type Status string

const (
	StatusNormal       Status = "normal"
	StatusMissingCL    Status = "missing_changelog"
	StatusMissingRel   Status = "missing_release"
	StatusOnlyTag      Status = "only_tag"
	StatusUnreleased   Status = "unreleased"
	StatusCausalBreak  Status = "causal_break"
)

type RepairAction struct {
	Scope   Scope  `json:"scope"`
	Type    string `json:"type"`
	Payload any    `json:"payload,omitempty"`
}

type ShelvedItem struct {
	Scope     Scope     `json:"scope"`
	Version   string    `json:"version"`
	Reason    string    `json:"reason"`
	ShelvedAt time.Time `json:"shelved_at"`
}

type ConvergeReport struct {
	Timestamp    time.Time    `json:"timestamp"`
	Total        int          `json:"total"`
	Normal       int          `json:"normal"`
	Fixed        int          `json:"fixed"`
	Shelved      int          `json:"shelved"`
	CausalBreaks int          `json:"causal_breaks"`
	Errors       int          `json:"errors"`
	Results      []ScanResult `json:"results,omitempty"`
}
