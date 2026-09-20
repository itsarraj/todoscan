use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;
use todoscan::git::blame_line_timestamp;
use todoscan::scan::find_markers;

const SKIP_DIR_NAMES: &[&str] = &[".git", "target", "node_modules", "vendor", "dist", "build"];

#[derive(Parser)]
#[command(
    name = "todoscan",
    about = "Scans a codebase for TODO/FIXME/HACK/XXX comments, oldest first by real git-blame age"
)]
struct Cli {
    /// Git repository root to scan.
    #[arg(default_value = ".")]
    dir: PathBuf,
}

struct Finding {
    path: PathBuf,
    line: usize,
    marker: &'static str,
    text: String,
    age_days: i64,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

    let mut files = Vec::new();
    walk(&cli.dir, &mut files)?;

    let mut findings = Vec::new();
    for path in &files {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue, // binary or unreadable file — skipped, not fatal
        };
        for hit in find_markers(&content) {
            let age_days = match blame_line_timestamp(&cli.dir, path, hit.line) {
                Ok(ts) => (now - ts) / 86_400,
                Err(_) => continue, // not a tracked line (uncommitted, or blame failed) — skip rather than guess
            };
            findings.push(Finding {
                path: path.clone(),
                line: hit.line,
                marker: hit.marker,
                text: hit.text,
                age_days,
            });
        }
    }

    findings.sort_by_key(|f| std::cmp::Reverse(f.age_days));

    if findings.is_empty() {
        println!("todoscan: no TODO/FIXME/HACK/XXX markers found");
        return Ok(());
    }

    for f in &findings {
        println!(
            "{} day(s) old  {}:{}  [{}]  {}",
            f.age_days,
            f.path.display(),
            f.line,
            f.marker,
            f.text
        );
    }
    println!("\n{} marker(s) found", findings.len());
    Ok(())
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            if SKIP_DIR_NAMES.contains(&name.to_string_lossy().as_ref()) {
                continue;
            }
            walk(&path, out)?;
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}
