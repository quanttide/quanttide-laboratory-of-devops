package main

import (
	"context"
	"net/http"

	"github.com/google/go-github/v88/github"
)

type GitHubClient struct {
	client *github.Client
}

func NewGitHubClient(httpClient *http.Client) (*GitHubClient, error) {
	c, err := github.NewClient(github.WithHTTPClient(httpClient))
	if err != nil {
		return nil, err
	}
	return &GitHubClient{client: c}, nil
}

func NewGitHubClientWithTransport(transport http.RoundTripper) (*GitHubClient, error) {
	c, err := github.NewClient(github.WithHTTPClient(&http.Client{Transport: transport}))
	if err != nil {
		return nil, err
	}
	return &GitHubClient{client: c}, nil
}

func (c *GitHubClient) ListTags(ctx context.Context, owner, repo string) ([]*github.Reference, error) {
	refs, _, err := c.client.Git.ListMatchingRefs(ctx, owner, repo, "refs/tags/")
	return refs, err
}

func (c *GitHubClient) ListDir(ctx context.Context, owner, repo, dir string) ([]string, error) {
	_, entries, _, err := c.client.Repositories.GetContents(ctx, owner, repo, dir, &github.RepositoryContentGetOptions{})
	if err != nil {
		return nil, err
	}
	var names []string
	for _, e := range entries {
		if e.GetType() == "dir" {
			names = append(names, e.GetName())
		}
	}
	return names, nil
}

func (c *GitHubClient) GetChangelog(ctx context.Context, owner, repo string) (string, error) {
	return c.GetFile(ctx, owner, repo, "CHANGELOG.md")
}

func (c *GitHubClient) GetFile(ctx context.Context, owner, repo, path string) (string, error) {
	content, _, _, err := c.client.Repositories.GetContents(ctx, owner, repo, path, &github.RepositoryContentGetOptions{})
	if err != nil {
		return "", err
	}
	decoded, err := content.GetContent()
	if err != nil {
		return "", err
	}
	return decoded, nil
}

func (c *GitHubClient) ListReleases(ctx context.Context, owner, repo string) ([]*github.RepositoryRelease, error) {
	releases, _, err := c.client.Repositories.ListReleases(ctx, owner, repo, &github.ListOptions{PerPage: 100})
	return releases, err
}

func (c *GitHubClient) CreateRelease(ctx context.Context, owner, repo, tag, body string) error {
	_, _, err := c.client.Repositories.CreateRelease(ctx, owner, repo, &github.RepositoryRelease{
		TagName: &tag,
		Body:    &body,
	})
	return err
}

func (c *GitHubClient) CreatePR(ctx context.Context, owner, repo, title, body, head, base string) error {
	_, _, err := c.client.PullRequests.Create(ctx, owner, repo, &github.NewPullRequest{
		Title: &title,
		Body:  &body,
		Head:  &head,
		Base:  &base,
	})
	return err
}
