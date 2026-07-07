package main

type JudgeResult struct {
	Status     Status
	Summary    string
	Repairable bool
	// FactSource indicates the hierarchy level for repair caution:
	// "tag" (不可移动), "changelog" (规范事实源), "release" (派生制品)
	FactSource string
}

var verdictTable = []struct {
	HasTag, HasCL, HasRel   bool
	TagCLMatch, TagRelMatch bool
	Status                  Status
	Summary                 string
	Repairable              bool
	FactSource              string
}{
	{true, true, true, true, true, StatusNormal, "artifact 完整，版本一致", false, ""},
	{true, true, true, true, false, StatusCausalBreak, "因果断裂：Release tag 不匹配已有 tag", false, "release"},
	{true, true, true, false, true, StatusCausalBreak, "因果断裂：CHANGELOG 版本与 tag 不匹配", false, "changelog"},
	{true, true, true, false, false, StatusCausalBreak, "因果断裂：CHANGELOG 和 Release 均与 tag 不匹配", false, "manual"},

	{true, false, true, false, true, StatusMissingCL, "缺 CHANGELOG", true, "changelog"},
	{true, false, true, false, false, StatusCausalBreak, "因果断裂：Release tag 不匹配且缺 CHANGELOG", false, "manual"},

	{true, true, false, true, false, StatusMissingRel, "缺 Release", true, "release"},
	{true, true, false, false, false, StatusCausalBreak, "因果断裂：CHANGELOG 版本与 tag 不匹配且缺 Release", false, "changelog"},

	{true, false, false, false, false, StatusOnlyTag, "只有 tag，缺 CHANGELOG 和 Release", true, "tag"},

	{false, false, false, false, false, StatusUnreleased, "未发布", false, ""},
	{false, true, true, false, false, StatusCausalBreak, "因果断裂：有 CHANGELOG 和 Release 但无 tag", false, "manual"},
	{false, true, false, false, false, StatusCausalBreak, "因果断裂：有 CHANGELOG 但无 tag （规范事实源悬空）", false, "manual"},
	{false, false, true, false, false, StatusCausalBreak, "因果断裂：有 Release 但无 tag （派生制品悬空）", false, "manual"},
}

func Judge(state ArtifactState) JudgeResult {
	for _, v := range verdictTable {
		if v.HasTag == state.HasTag &&
			v.HasCL == state.HasChangelog &&
			v.HasRel == state.HasRelease &&
			v.TagCLMatch == state.TagCLMatch &&
			v.TagRelMatch == state.TagRelMatch {
			return JudgeResult{
				Status:     v.Status,
				Summary:    v.Summary,
				Repairable: v.Repairable,
				FactSource: v.FactSource,
			}
		}
	}
	return JudgeResult{Status: StatusUnreleased, Summary: "未知状态", Repairable: false}
}

type Stats struct {
	Total        int `json:"total"`
	Normal       int `json:"normal"`
	Abnormal     int `json:"abnormal"`
	Shelved      int `json:"shelved"`
	CausalBreaks int `json:"causal_breaks"`
}

func Aggregate(results []ScanResult) Stats {
	s := Stats{Total: len(results)}
	for _, r := range results {
		switch r.Status {
		case StatusNormal:
			s.Normal++
		case StatusOnlyTag:
			s.Shelved++
		case StatusCausalBreak:
			s.CausalBreaks++
			s.Abnormal++
		default:
			s.Abnormal++
		}
	}
	return s
}
