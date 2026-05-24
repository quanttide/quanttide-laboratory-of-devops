use std::path::Path;

pub fn run(repo_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let files = [
        ("BUGS.md", "已知缺陷"),
        ("ROADMAP.md", "迭代计划"),
        ("TODO.md", "待办事项"),
        ("CHANGELOG.md", "变更记录"),
    ];

    let mut total_lines = 0usize;
    let mut total_todos = 0usize;

    println!("项目规划摘要");
    println!("{}", "-".repeat(40));

    for (filename, label) in &files {
        let path = repo_path.join(filename);
        if !path.exists() {
            println!("  {}: 不存在", filename);
            continue;
        }
        let content = std::fs::read_to_string(&path)?;
        let lines = content.lines().count();
        total_lines += lines;

        let todo_count = content.lines().filter(|l| l.trim().starts_with("- [ ]")).count();
        let done_count = content.lines().filter(|l| l.trim().starts_with("- [x]")).count();
        total_todos += todo_count + done_count;
        println!("  {} ({}): {} 行", filename, label, lines);
        if todo_count + done_count > 0 {
            println!("    待办: {} 完成: {}", todo_count, done_count);
        }
    }

    println!();
    println!("  总计: {} 文件, {} 行", files.len(), total_lines);
    if total_todos > 0 {
        println!("  待办项: {}", total_todos);
    }

    Ok(total_lines.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_file(dir: &std::path::Path, name: &str, content: &str) {
        std::fs::write(dir.join(name), content).unwrap();
    }

    #[test]
    fn test_plan_no_files() {
        let dir = tempfile::tempdir().unwrap();
        run(dir.path()).unwrap();
    }

    #[test]
    fn test_plan_with_todos() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "TODO.md", "# TODO\n- [ ] task 1\n- [x] task 2\n- [ ] task 3\n");
        write_file(dir.path(), "CHANGELOG.md", "# Changelog\n\n## [1.0.0]\n\ncontent\n");
        let result = run(dir.path()).unwrap();
        assert_eq!(result, "9");
    }

    #[test]
    fn test_plan_with_bugs() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "BUGS.md", "# BUGS\n- BUG: something broken\n");
        write_file(dir.path(), "ROADMAP.md", "# ROADMAP\n\n## Iter 1\n\nstuff\n");
        run(dir.path()).unwrap();
    }

    #[test]
    fn test_plan_reads_line_count() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "TODO.md", "a\nb\nc\n");
        let result = run(dir.path()).unwrap();
        assert_eq!(result, "3");
    }
}
