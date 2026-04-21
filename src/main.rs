use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const MARKERS: &[&str] = &["TODO", "FIXME", "HACK", "XXX"];

const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".build",
    "vendor",
    "dist",
    "__pycache__",
];

struct Finding {
    path: PathBuf,
    line: usize,
    marker: String,
    text: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = args.get(1).map(|s| s.as_str()).unwrap_or(".");
    let json = args.iter().any(|a| a == "--json");

    let mut findings: Vec<Finding> = Vec::new();
    walk(Path::new(dir), &mut findings);

    if json {
        print_json(&findings);
    } else {
        print_table(&findings);
    }

    if !findings.is_empty() {
        std::process::exit(0);
    }
}

fn walk(dir: &Path, findings: &mut Vec<Finding>) {
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

fn scan_file(path: &Path, findings: &mut Vec<Finding>) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

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

fn print_table(findings: &[Finding]) {
    if findings.is_empty() {
        println!("No TODOs found.");
        return;
    }

    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for f in findings {
        *counts.entry(&f.marker).or_default() += 1;
    }

    println!("Found {} items:", findings.len());
    for marker in MARKERS {
        if let Some(&count) = counts.get(marker) {
            println!("  {}: {}", marker, count);
        }
    }
    println!();

    for f in findings {
        println!(
            "  {}:{} [{}] {}",
            f.path.display(),
            f.line,
            f.marker,
            f.text
        );
    }
}

fn print_json(findings: &[Finding]) {
    print!("[");
    for (i, f) in findings.iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        let path = f.path.display().to_string().replace('\\', "/");
        let text = f.text.replace('\\', "\\\\").replace('"', "\\\"");
        print!(
            r#"{{"file":"{}","line":{},"marker":"{}","text":"{}"}}"#,
            path, f.line, f.marker, text
        );
    }
    println!("]");
}
