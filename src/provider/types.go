package main

import "time"

type Scope string

type ArtifactState struct {
	Scope         Scope  `json:"scope"`
	Owner         string `json:"owner"`
	Repo          string `json:"repo"`
	Version       string `json:"version"`
	HasTag        bool   `json:"has_tag"`
	HasChangelog  bool   `json:"has_changelog"`
	HasRelease    bool   `json:"has_release"`
}

type ScanResult struct {
	Scope      Scope    `json:"scope"`
	State      ArtifactState `json:"state"`
	Status     Status   `json:"status"`
	Summary    string   `json:"summary"`
	Repairable bool     `json:"repairable"`
}

type Status string

const (
	StatusNormal       Status = "normal"
	StatusMissingCL    Status = "missing_changelog"
	StatusMissingRel   Status = "missing_release"
	StatusOnlyTag      Status = "only_tag"
	StatusUnreleased   Status = "unreleased"
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
