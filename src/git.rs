//! Real `git blame` integration: how old is the specific line a marker
//! was found on. Shells out to the real `git` binary rather than
//! reimplementing blame — porcelain output is a stable, documented
//! format meant for exactly this kind of scripting.

use std::path::Path;
use std::process::Command;

/// The Unix timestamp (`author-time`, real commit-time seconds since
/// epoch) of the commit that last touched `file`'s `line` (1-indexed),
/// via `git blame -L line,line --porcelain`.
pub fn blame_line_timestamp(repo_root: &Path, file: &Path, line: usize) -> anyhow::Result<i64> {
    let output = Command::new("git")
        .arg("blame")
        .arg("-L")
        .arg(format!("{line},{line}"))
        .arg("--porcelain")
        .arg(file)
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "git blame failed for {}:{line}: {}",
            file.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    parse_author_time(&String::from_utf8_lossy(&output.stdout))
}

fn parse_author_time(porcelain: &str) -> anyhow::Result<i64> {
    for line in porcelain.lines() {
        if let Some(rest) = line.strip_prefix("author-time ") {
            return rest
                .trim()
                .parse()
                .map_err(|e| anyhow::anyhow!("malformed author-time in blame output: {e}"));
        }
    }
    anyhow::bail!("no author-time line found in blame output")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_author_time_from_realistic_porcelain_output() {
        let porcelain = "abc123 1 1 1\n\
author Jane Doe\n\
author-mail <jane@example.com>\n\
author-time 1700000000\n\
author-tz +0000\n\
committer Jane Doe\n\
\tsome line content\n";
        assert_eq!(parse_author_time(porcelain).unwrap(), 1_700_000_000);
    }

    #[test]
    fn missing_author_time_is_a_clean_error_not_a_panic() {
        assert!(parse_author_time("no relevant fields here\n").is_err());
    }

    #[test]
    fn malformed_author_time_value_is_a_clean_error() {
        assert!(parse_author_time("author-time not-a-number\n").is_err());
    }
}
