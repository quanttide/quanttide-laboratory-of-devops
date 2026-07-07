package main

type JudgeResult struct {
	Status     Status
	Summary    string
	Repairable bool
}

var decisionTable = map[[3]bool]JudgeResult{
	{true, true, true}:   {StatusNormal, "artifact 完整", false},
	{true, false, true}:  {StatusMissingCL, "缺 CHANGELOG", true},
	{true, true, false}:  {StatusMissingRel, "缺 Release", true},
	{true, false, false}: {StatusOnlyTag, "只有 tag，缺 CHANGELOG 和 Release", true},
	{false, false, false}: {StatusUnreleased, "未发布（无 tag/CHANGELOG/Release）", false},
	{false, true, true}:   {StatusUnreleased, "有 CHANGELOG 和 Release 但无 tag（异常）", false},
	{false, true, false}:  {StatusUnreleased, "有 CHANGELOG 但无 tag/Release（异常）", false},
	{false, false, true}:  {StatusUnreleased, "有 Release 但无 tag/CHANGELOG（异常）", false},
}

func Judge(state ArtifactState) JudgeResult {
	key := [3]bool{state.HasTag, state.HasChangelog, state.HasRelease}
	r, ok := decisionTable[key]
	if !ok {
		return JudgeResult{Status: StatusUnreleased, Summary: "未知状态", Repairable: false}
	}
	return r
}

type Stats struct {
	Total    int `json:"total"`
	Normal   int `json:"normal"`
	Abnormal int `json:"abnormal"`
	Shelved  int `json:"shelved"`
}

func Aggregate(results []ScanResult) Stats {
	s := Stats{Total: len(results)}
	for _, r := range results {
		switch r.Status {
		case StatusNormal:
			s.Normal++
		case StatusOnlyTag:
			s.Shelved++
		default:
			s.Abnormal++
		}
	}
	return s
}
