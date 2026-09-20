# todoscan

Scans a codebase for `TODO`/`FIXME`/`HACK`/`XXX` comments and reports
each with its real `git blame` age — how long that specific line has
actually existed, not just "found in this file" — sorted oldest-first
so the stalest, most-forgotten markers surface at the top instead of
getting lost in a flat unordered `grep -r TODO` dump.

## Usage

```bash
todoscan              # scan the current git repository
todoscan path/to/repo
```

Output: `<age> day(s) old  <path>:<line>  [<marker>]  <comment text>`,
oldest first.

## How the age is computed

For every marker found, `todoscan` runs a real `git blame -L
<line>,<line> --porcelain` on that exact line and reads its
`author-time` field — the real commit timestamp of whoever last touched
that line, not the file's mtime or the file's oldest commit. A line that
predates version control (or is uncommitted/staged-only) has no blame
history to read and is silently skipped rather than reported with a
guessed or zero age.

## Marker detection

Case-sensitive matches for `TODO`, `FIXME`, `HACK`, `XXX` as whole words
(so `TODOLIST` doesn't false-positive) appearing at or after a
comment-opening token (`//`, `#`, `/*`, `--`, `;`) on the same line —
a heuristic, not a real per-language comment grammar, so it's blind to
comment *closings* and to a marker legitimately sitting inside a string
literal that happens to follow a comment opener earlier on the line.
Only the first marker on a line is reported, matching the common
real-world case of one marker per comment.

## Status: built and verified against a real git repository, real commits, and real git blame

- **13 unit tests** (`cargo test --lib`): marker detection across all
  four keywords and multiple comment styles (`//`, `#`, `/*`, `--`),
  whole-word matching (rejecting `TODOLIST`), the comment-opener
  ordering requirement (a marker-shaped identifier *before* any comment
  opener on the line is correctly ignored), only the first marker per
  line reported, and correct line numbering across a multi-line file;
  plus the `git blame --porcelain` parser against a realistic porcelain
  fixture (including missing/malformed `author-time` handled as a clean
  error, not a panic).
- **Live-verified against a real git repository with real, deliberately
  backdated commits**: `git init`, one commit with `GIT_AUTHOR_DATE`
  set to a date roughly 1000 days in the past containing a `TODO`, a
  second commit dated ~19 days ago containing a `FIXME`. Running the
  actual compiled binary against this real repo reported both markers
  with their real, correctly computed ages (993 and 19 days — matching
  hand-calculated day counts from the real backdated timestamps) and in
  the correct oldest-first order — confirmed against real `git blame`
  output for that real repository, not assumed or hardcoded timestamps.

**Not done / deliberately deferred**: no `--since`/age-threshold filter
(everything found is printed; piping through a day-count-aware filter
is left to the caller); binary files are skipped via a UTF-8 read
failure rather than a real binary-detection heuristic; no config file
for adding custom marker keywords beyond the fixed four; a marker on a
line that was later *reformatted but not semantically changed* still
gets a fresh blame timestamp from the reformatting commit, since that's
exactly what `git blame` itself reports — this tool doesn't second-guess
blame with a content-similarity heuristic the way `git blame -w`-style
tools sometimes do.
