# AGENTS.md

## Coding Standards

- **Spelling:** Must use Canadian Spelling for things that don't require American spelling (e.g., UI strings, variables, comments). Examples: "colour", "center" -> "centre", "behavior" -> "behaviour".
- **Commit Messages:** Use Conventional Commits (e.g., `feat: add scanner`, `fix: typo in header`).

## Commit Messages

**Format:** Use Conventional Commits format (e.g., `feat: ...`, `fix: ...`, `docs: ...`, `test: ...`).

- Use `test:` for test-related changes, including fixes to tests themselves (do not use `fix:` unless it fixes application code).

**Body Requirements:**

- Explain what and why (not how)
- Use markdown: **bold**, _italics_, `code`, bullet lists
- **NO markdown headings** - use **bold labels** for sections (not always required)
- When a commit body includes backticked code in shell commands, avoid command substitution by using single-quoted `-m` strings (preferred) or escaping backticks.
  - Example (preferred): `git commit -m 'fix: ...' -m 'Use `scanStatus` in footer'`
  - Example (escape): `git commit -m "Use \`scanStatus\` in footer"`

**Specific Updates**: Each commit message should reflect the specific changes made in that commit. Do not just recap the entire project history or scope. Focus on the now.

**Shell Interpolation Safety:**

- Do not pass markdown-heavy commit bodies directly via `git commit -m "..."` when they include backticks, `$()`, or shell-sensitive characters.
- Prefer writing the message to a file with a single-quoted heredoc and commit with `git commit -F <file>` to prevent shell expansion.
- If using `-m`, escape shell-sensitive characters explicitly before running the command.
- After committing, verify the stored message with `git log -1 --pretty=fuller` and amend immediately if interpolation altered content.

## Pull Request Titles

**Requirement:** PR titles must be human-readable summaries of the PR change.

- Start with a capital letter.
- Do not use Conventional Commit prefixes in PR titles (for example, no `feat:`, `fix:`, `chore:`).
- Describe the outcome or behaviour change, not internal process language.
- Ignore internal planning document notes in PR titles and descriptions unless they directly map to repository changes.
- Keep title style consistent across every open PR in the same stack.
- If one title in a stack is updated, update the rest of the open stack titles to match style and scope.
- Do not rename merged PRs unless explicitly requested.
- Keep linked issues and merge order aligned after any title changes in a stack.

## Pull Request Content

**Requirement:** PR titles and descriptions must not mention internal workflow artefacts.

- Do not mention deferred-review documents, internal queue labels, or internal-only planning notes in outward PR content.
- Keep internal triage mechanics in local runbooks, internal labels, and agent workflows only.
- Use user-facing, outcome-focused language in PR titles and descriptions.
- Only include internal process details in PR content when explicitly requested by the user.

## Pull Request Labels

**Requirement:** Every PR must include labels that describe the change and map to release-note categories.

- Add at least one primary category label to every PR: `enhancement`, `bug`, `documentation`, `testing`, `ci`, `build`, or `chore`.
- Add shared operational labels where they help clarify handling: `infrastructure`, `internal`, `release`, `blocked`, `epic`, or `skip-changelog`.
- Add product and subsystem scope labels where helpful (for example, `cli`, `tui`, `core`, `store`, `context`, `inference`, `model`).
- Prefer the broader Liminal HQ label style over Conventional Commit terms for PR labelling. Use GitHub label categories like `enhancement` and `bug` instead of labels such as `feat` or `fix`.
- Use `skip-changelog` only when a change should be excluded from generated release notes.
- Keep labels accurate as scope changes during review.

## Git Workflow

**Requirement:** Do not push changes (especially force pushes) to the repository unless explicitly requested by the user.

## Testing

- **Mandatory Testing:** Make sure the unit tests are run after changes to the code.
- **Verification:** Always verify code changes by running relevant tests.
- **Build Check:** Run `cargo build` and `cargo clippy --workspace -- -D warnings` to surface any errors.
- **Format Check:** Run `cargo fmt --check` to ensure consistent formatting.
- **Test Commands:**
  - `cargo test` — run all workspace tests
  - `cargo test -p liminal-flow-core` — core tests only
  - `cargo test -p liminal-flow-store` — store tests only

## Documentation

- **Updates:** When user-facing behaviour, CLI options, or features change, update `README.md` and `SPEC.md`.

## Project Structure

- This is a Rust project (2021 edition) using a Cargo workspace with five crates.
- The binary is `flo` (built from `liminal-flow-cli`).
- TUI uses ratatui + crossterm + tui-textarea.
- CLI parsing uses clap 4 with derive macros.
- Persistence uses rusqlite (bundled) with SQLite in WAL mode.

### Crate Layout

| Crate | Purpose |
|---|---|
| `liminal-flow-core` | Domain model, events, reducer, rules |
| `liminal-flow-store` | SQLite persistence, migrations, repositories |
| `liminal-flow-cli` | CLI entrypoint and command handlers |
| `liminal-flow-tui` | Terminal UI (ratatui) |
| `liminal-flow-context` | Git and workspace context discovery |

## Licence and Copyright

- **Requirement:** New source files (and substantially rewritten source files) should include a short header as the first content in the file.
- **Applies to:** `.rs` source files in `src/` directories.
- **Do not add headers to:** generated files, lockfiles, config files (`.json`, `.yml`, `.toml`), markdown docs, or man pages.

Preferred header format for Rust:

```rust
// Brief one-line summary of what this file does
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT
```

- Keep the summary to one concise sentence.
- Place the header before `use` statements.
- Leave one blank line between the header and the first code line.
- Preserve existing valid licence headers when already present.
