package main

type JudgeResult struct {
	Status     Status
	Summary    string
	Repairable bool
	FactSource string
}

type verdictRow struct {
	HasTag, HasCL, HasRel  bool
	CLCmp                  CmpResult // only checked when both tag and CL exist
	TagRelMatch            bool
	Status                 Status
	Summary                string
	Repairable             bool
	FactSource             string
}

var verdictTable = []verdictRow{
	// ── 三者完整 ──────────────────────────────────────────────────
	{true, true, true, CmpEqual, true, StatusNormal, "artifact 完整，版本一致", false, ""},
	{true, true, true, CmpLess, true, StatusPendingRel, "CL 超前 tag，待发版", true, "tag"},
	{true, true, true, CmpGreater, true, StatusCausalBreak, "CHANGELOG 落后于 tag", false, "changelog"},
	{true, true, true, CmpEqual, false, StatusCausalBreak, "Release tag 不匹配已有 tag", false, "release"},
	{true, true, true, CmpLess, false, StatusPendingRel, "CL 超前，Release tag 不匹配", true, "tag"},
	{true, true, true, CmpGreater, false, StatusCausalBreak, "CHANGELOG 落后，Release 也不匹配", false, "manual"},

	// ── 缺 CHANGELOG ──────────────────────────────────────────────
	{true, false, true, CmpEqual, true, StatusMissingCL, "缺 CHANGELOG", true, "changelog"},
	{true, false, true, CmpEqual, false, StatusCausalBreak, "缺 CHANGELOG 且 Release tag 不匹配", false, "manual"},

	// ── 缺 Release ───────────────────────────────────────────────
	{true, true, false, CmpEqual, false, StatusMissingRel, "缺 Release", true, "release"},
	{true, true, false, CmpLess, false, StatusPendingRel, "CL 超前，缺 Release", true, "tag"},
	{true, true, false, CmpGreater, false, StatusCausalBreak, "CHANGELOG 落后于 tag，缺 Release", false, "manual"},

	// ── 只有 tag ──────────────────────────────────────────────────
	{true, false, false, CmpEqual, false, StatusOnlyTag, "只有 tag，缺 CHANGELOG 和 Release", true, "tag"},

	// ── 无 tag ────────────────────────────────────────────────────
	{false, false, false, CmpEqual, false, StatusUnreleased, "未发布", false, ""},
	{false, true, true, CmpEqual, false, StatusCausalBreak, "有 CHANGELOG 和 Release 但无 tag（因果矛盾）", false, "manual"},
	{false, true, false, CmpEqual, false, StatusCausalBreak, "有 CHANGELOG 但无 tag（规范事实源悬空）", false, "manual"},
	{false, false, true, CmpEqual, false, StatusCausalBreak, "有 Release 但无 tag（派生制品悬空）", false, "manual"},
}

func Judge(state ArtifactState) JudgeResult {
	cmp := state.TagCLCmp
	if !state.HasTag || !state.HasChangelog {
		cmp = CmpEqual // no comparison possible
	}

	for _, v := range verdictTable {
		if v.HasTag == state.HasTag &&
			v.HasCL == state.HasChangelog &&
			v.HasRel == state.HasRelease &&
			v.CLCmp == cmp &&
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
	PendingRel   int `json:"pending_release"`
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
		case StatusPendingRel:
			s.PendingRel++
			s.Abnormal++
		default:
			s.Abnormal++
		}
	}
	return s
}
