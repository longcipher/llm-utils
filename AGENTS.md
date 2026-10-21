# llm-utils Agent Instructions

## Non-negotiables

- Surgical edits only: every changed line must trace to the request. No drive-by refactors, no reformatting untouched code, no deleting pre-existing dead code (mention it instead).
- Fix root causes. Never silence a warning, weaken a check, or patch a symptom to make a gate pass.
- Never report completion while any `just` gate below fails. Report the failure verbatim and stop.
- Stop and ask when a request is ambiguous, conflicts with existing conventions, or requires credentials, publishing, or destructive operations.
- When a correction reveals a missing rule, add one specific rule here ("always X for Y", never "be careful with Y"); delete rules that never catch real errors.

## Scope

- llm-utils is a small Rust utility workspace for preparing text for LLM prompts. It has no binaries, no network calls, no database and no async runtime.
- `crates/llm-text` is the text crate: whitespace/citation normalization (`text`), HTML-to-text extraction via readability (`html`), URL extraction and deduplication (`links`), and sentence-level plus recursive splitting (`splitting`).
- `crates/opml-protocol` is the OPML 2.0 crate: parsing and generation through `quick-xml` + `serde`, a `derive_builder` document builder, and RSS feed extraction grouped by folder hierarchy.
- The workspace has exactly two members (`crates/*`). `bin/` exists but is empty; do not add a binary crate without an agreed CLI surface.
- `crates/llm-text/README.md` and `crates/opml-protocol/README.md` are the published crate docs and must stay in sync with the public API.
- English only in code, comments, docs, and commit messages.

## Execution

- Parallelize aggressively: dispatch subagents for independent research, exploration, and implementation.
- NEVER run two `cargo` commands concurrently, and never overlap file edits with a running `cargo` command (`target/` file lock): finish edits, wait, then run.
- NEVER use `git worktree`. Edit the current branch in place.
- TDD for behavior changes: failing crate-local test first, smallest change to pass, then run `just mutation` and fix surviving mutants.

## Verification loop

After each feature or bug fix, in order; any failure means the task is not done:

```bash
just format
just lint
just test
just mutation
```

`just lint` is exactly the tool set CI installs: `typos`, `rumdl`, `taplo fmt --check`, `cargo +nightly fmt --check`, `cargo +nightly clippy --all -- -D warnings`, `cargo machete`.

## Workspace rules

- Never hand-type dependency versions. Add per-crate deps with `cargo add <crate> -p <crate>`. Shared deps go into root `[workspace.dependencies]` (numeric versions only, no default features) and are referenced with `dep = { workspace = true }`.
- Sub-crates inherit `version`/`edition`/`license`/`repository` via `workspace = true` and must set `[lints] workspace = true`.
- TOML formatting and ordering is `taplo` (CI installs `taplo-cli`, not `cargo-sort`); `.taplo.toml` already defines the dependency-block ordering rules. Do not introduce `cargo sort` here.
- Unused-dependency detection is `cargo machete` (CI installs `cargo-machete`, not `cargo-shear`). Do not introduce `cargo shear` here.
- There is no `rust-toolchain.toml`; CI installs a nightly toolchain, so use `cargo +nightly fmt` and `cargo +nightly clippy` exactly as the Justfile does.
- `allow_attributes` is `deny`: a bare `#[allow(...)]` is a hard compile error. The same is true for `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `print_stdout` and `print_stderr`.
- There is no property-test framework wired up. `proptest` sits unused in `[workspace.dependencies]`; if you add property tests, add it to the crate that owns the logic so it runs in the ordinary `cargo test` path.
- `just bdd` is a placeholder: no BDD harness is configured. Do not add scenario runners without an agreed format.

## Dependency choices

- Errors: `thiserror` in `opml-protocol` (concrete `OpmlError` with `#[from]` conversions for `quick_xml` errors), `eyre` in `llm-text` where the API returns `Result`. Never `anyhow`.
- HTML extraction: `dom_smoothie` plus `html2text`. URL extraction: `linkify` plus `url`. Text segmentation: `unicode-segmentation`. All four are already workspace dependencies.
- XML: `quick-xml` with the `serialize` feature plus `serde`; do not hand-roll an XML parser or add a second XML crate.
- Struct construction: `derive_builder` for the OPML builder types.
- Regex: `regex` only. No `reqwest`, no `tokio`, no `sqlx`, no `dashmap`, no `log`/`tracing` (these crates do not log).

## Code conventions

- Public API is the crate's contract: every public item needs a doc comment, and `missing_docs` plus `missing_debug_implementations` warn at workspace level.
- `opml-protocol` keeps a process-wide `ATTR_KEY_CACHE` (`LazyLock<Mutex<HashMap<String, &'static str>>>`) so dynamic attribute keys can be handed to `serde` as `&'static str`. It recovers from a poisoned lock with `unwrap_or_else(|e| e.into_inner())`; keep that pattern instead of unwrapping.
- That cache leaks strings on purpose. Do not change its growth behaviour without a stated bound.
- Parsing must be total: malformed input returns an `OpmlError`, it never panics and never silently drops a feed.
- Tests are example-based and colocated in `#[cfg(test)]` modules; integration tests belong in `crates/*/tests/`.
- `crates/llm-text/src/text.rs` holds a deliberate multilingual fixture (Latin, Cyrillic and CJK characters) that asserts non-ASCII text survives unless `remove_non_basic_ascii()` is opted into; `just check-cn` excludes that file for the same reason.
