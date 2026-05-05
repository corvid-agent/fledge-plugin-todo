use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const MARKERS: &[&str] = &["TODO", "FIXME", "HACK", "XXX"];

pub const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".build",
    "vendor",
    "dist",
    "__pycache__",
];

#[derive(Debug, PartialEq)]
pub struct Finding {
    pub path: PathBuf,
    pub line: usize,
    pub marker: String,
    pub text: String,
}

pub fn walk(dir: &Path, findings: &mut Vec<Finding>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !SKIP_DIRS.contains(&name.as_ref()) {
                walk(&path, findings);
            }
        } else if path.is_file() {
            scan_file(&path, findings);
        }
    }
}

pub fn scan_file(path: &Path, findings: &mut Vec<Finding>) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    scan_content(path, &content, findings);
}

pub fn scan_content(path: &Path, content: &str, findings: &mut Vec<Finding>) {
    for (i, line) in content.lines().enumerate() {
        for marker in MARKERS {
            if let Some(pos) = line.find(marker) {
                let rest = line[pos + marker.len()..].trim_start_matches([':', ' ', '(']);
                let text = rest.trim().to_string();
                findings.push(Finding {
                    path: path.to_path_buf(),
                    line: i + 1,
                    marker: marker.to_string(),
                    text,
                });
                break;
            }
        }
    }
}

pub fn format_table(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "No TODOs found.\n".to_string();
    }

    let mut out = String::new();
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for f in findings {
        *counts.entry(&f.marker).or_default() += 1;
    }

    out.push_str(&format!("Found {} items:\n", findings.len()));
    for marker in MARKERS {
        if let Some(&count) = counts.get(marker) {
            out.push_str(&format!("  {}: {}\n", marker, count));
        }
    }
    out.push('\n');

    for f in findings {
        out.push_str(&format!(
            "  {}:{} [{}] {}\n",
            f.path.display(),
            f.line,
            f.marker,
            f.text
        ));
    }
    out
}

pub fn format_json(findings: &[Finding]) -> String {
    let mut out = String::from("[");
    for (i, f) in findings.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let path = f.path.display().to_string().replace('\\', "/");
        let text = f.text.replace('\\', "\\\\").replace('"', "\\\"");
        out.push_str(&format!(
            r#"{{"file":"{}","line":{},"marker":"{}","text":"{}"}}"#,
            path, f.line, f.marker, text
        ));
    }
    out.push_str("]\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn scan_content_finds_todo() {
        let mut findings = Vec::new();
        let content = "let x = 1; // TODO: fix this later\n";
        scan_content(Path::new("test.rs"), content, &mut findings);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].marker, "TODO");
        assert_eq!(findings[0].text, "fix this later");
        assert_eq!(findings[0].line, 1);
    }

    #[test]
    fn scan_content_finds_fixme() {
        let mut findings = Vec::new();
        let content = "// FIXME: broken on Windows\n";
        scan_content(Path::new("lib.rs"), content, &mut findings);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].marker, "FIXME");
        assert_eq!(findings[0].text, "broken on Windows");
    }

    #[test]
    fn scan_content_finds_hack() {
        let mut findings = Vec::new();
        let content = "# HACK workaround for upstream bug\n";
        scan_content(Path::new("script.py"), content, &mut findings);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].marker, "HACK");
        assert_eq!(findings[0].text, "workaround for upstream bug");
    }

    #[test]
    fn scan_content_finds_xxx() {
        let mut findings = Vec::new();
        let content = "// XXX: needs review\n";
        scan_content(Path::new("foo.js"), content, &mut findings);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].marker, "XXX");
        assert_eq!(findings[0].text, "needs review");
    }

    #[test]
    fn scan_content_multiple_markers() {
        let mut findings = Vec::new();
        let content = "// TODO: first\nlet x = 1;\n// FIXME: second\n";
        scan_content(Path::new("f.rs"), content, &mut findings);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].marker, "TODO");
        assert_eq!(findings[0].line, 1);
        assert_eq!(findings[1].marker, "FIXME");
        assert_eq!(findings[1].line, 3);
    }

    #[test]
    fn scan_content_no_markers() {
        let mut findings = Vec::new();
        let content = "fn main() {\n    println!(\"hello\");\n}\n";
        scan_content(Path::new("clean.rs"), content, &mut findings);
        assert!(findings.is_empty());
    }

    #[test]
    fn scan_content_marker_without_colon() {
        let mut findings = Vec::new();
        let content = "// TODO implement caching\n";
        scan_content(Path::new("f.rs"), content, &mut findings);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].text, "implement caching");
    }

    #[test]
    fn scan_content_only_first_marker_per_line() {
        let mut findings = Vec::new();
        // Line has both TODO and FIXME; only TODO should match (break after first)
        let content = "// TODO: fix this FIXME later\n";
        scan_content(Path::new("f.rs"), content, &mut findings);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].marker, "TODO");
    }

    #[test]
    fn format_table_empty() {
        let output = format_table(&[]);
        assert_eq!(output, "No TODOs found.\n");
    }

    #[test]
    fn format_table_with_findings() {
        let findings = vec![
            Finding {
                path: PathBuf::from("src/main.rs"),
                line: 10,
                marker: "TODO".to_string(),
                text: "add tests".to_string(),
            },
            Finding {
                path: PathBuf::from("src/lib.rs"),
                line: 20,
                marker: "FIXME".to_string(),
                text: "broken".to_string(),
            },
        ];
        let output = format_table(&findings);
        assert!(output.contains("Found 2 items:"));
        assert!(output.contains("TODO: 1"));
        assert!(output.contains("FIXME: 1"));
        assert!(output.contains("src/main.rs:10 [TODO] add tests"));
        assert!(output.contains("src/lib.rs:20 [FIXME] broken"));
    }

    #[test]
    fn format_json_empty() {
        let output = format_json(&[]);
        assert_eq!(output, "[]\n");
    }

    #[test]
    fn format_json_with_findings() {
        let findings = vec![Finding {
            path: PathBuf::from("src/main.rs"),
            line: 5,
            marker: "TODO".to_string(),
            text: "do something".to_string(),
        }];
        let output = format_json(&findings);
        assert!(output.starts_with('['));
        assert!(output.contains(r#""file":"src/main.rs""#));
        assert!(output.contains(r#""line":5"#));
        assert!(output.contains(r#""marker":"TODO""#));
        assert!(output.contains(r#""text":"do something""#));
    }

    #[test]
    fn format_json_escapes_quotes() {
        let findings = vec![Finding {
            path: PathBuf::from("f.rs"),
            line: 1,
            marker: "TODO".to_string(),
            text: r#"handle "edge" case"#.to_string(),
        }];
        let output = format_json(&findings);
        assert!(output.contains(r#"\"edge\""#));
    }

    #[test]
    fn walk_skips_git_dir() {
        let tmp = std::env::temp_dir().join("fledge_todo_test_skip_git");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(tmp.join(".git/config"), "// TODO: should be skipped\n").unwrap();
        fs::write(tmp.join("visible.rs"), "// TODO: should be found\n").unwrap();

        let mut findings = Vec::new();
        walk(&tmp, &mut findings);

        assert_eq!(findings.len(), 1);
        assert!(findings[0].path.ends_with("visible.rs"));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn walk_skips_node_modules() {
        let tmp = std::env::temp_dir().join("fledge_todo_test_skip_nm");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("node_modules/pkg")).unwrap();
        fs::write(tmp.join("node_modules/pkg/index.js"), "// TODO: hidden\n").unwrap();
        fs::write(tmp.join("app.js"), "// FIXME: visible\n").unwrap();

        let mut findings = Vec::new();
        walk(&tmp, &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].marker, "FIXME");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn scan_file_skips_binary() {
        let tmp = std::env::temp_dir().join("fledge_todo_test_binary");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        // Write non-UTF-8 bytes
        fs::write(tmp.join("binary.dat"), &[0xFF, 0xFE, 0x00, 0x01]).unwrap();

        let mut findings = Vec::new();
        scan_file(&tmp.join("binary.dat"), &mut findings);
        assert!(findings.is_empty());

        let _ = fs::remove_dir_all(&tmp);
    }
}
