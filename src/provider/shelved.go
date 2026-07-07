package main

import (
	"encoding/json"
	"os"
	"sync"
)

type ShelvedStore struct {
	mu   sync.Mutex
	path string
}

func NewShelvedStore(path string) *ShelvedStore {
	return &ShelvedStore{path: path}
}

func (s *ShelvedStore) Append(item ShelvedItem) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	var items []ShelvedItem
	data, err := os.ReadFile(s.path)
	if err == nil {
		json.Unmarshal(data, &items)
	}
	items = append(items, item)
	out, err := json.MarshalIndent(items, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(s.path, out, 0644)
}
