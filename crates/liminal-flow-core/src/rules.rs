// Deterministic interpretation rules and title normalisation for v1
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use crate::model::Intent;

/// Normalise a raw title by trimming whitespace and stripping common leading phrases.
///
/// Examples:
///   "I'm improving AIDX" → "improving AIDX"
///   "  working on docs  " → "docs"
///   "AIDX" → "AIDX" (preserved as-is)
pub fn normalise_title(raw: &str) -> String {
    let trimmed = raw.trim();

    // Strip common leading phrases (case-insensitive)
    // Longest prefixes first to avoid partial matches
    let prefixes = [
        "i'm working on ",
        "i am working on ",
        "i'm ",
        "i am ",
        "im ",
        "working on ",
    ];

    let lower = trimmed.to_lowercase();
    for prefix in &prefixes {
        if lower.starts_with(prefix) {
            let rest = &trimmed[prefix.len()..];
            let result = rest.trim();
            if !result.is_empty() {
                return result.to_string();
            }
        }
    }

    trimmed.to_string()
}

/// Slash command definitions for the parser.
///
/// Each entry maps a command name to its intent, whether a non-empty argument
/// is required, and whether trailing text is accepted at all.
///
/// - `requires_arg = true`: a non-empty argument is mandatory (e.g. `/now`)
/// - `accepts_trailing = true`: optional trailing text is kept (e.g. `/done shipped`)
/// - `accepts_trailing = false`: exact match only (e.g. `/where`)
const COMMAND_TABLE: &[(&str, Intent, bool, bool)] = &[
    //  command      intent                     requires  accepts_trailing
    ("/now", Intent::SetCurrentThread, true, true),
    ("/branch", Intent::StartBranch, true, true),
    ("/back", Intent::ReturnToParent, false, true),
    ("/note", Intent::AddNote, true, true),
    ("/where", Intent::QueryCurrent, false, false),
    ("/resume", Intent::Resume, false, true),
    ("/pause", Intent::Pause, false, true),
    ("/park", Intent::Park, false, true),
    ("/done", Intent::Done, false, true),
    ("/archive", Intent::Archive, false, true),
];

/// Detect the intent of a slash command from TUI input.
///
/// Returns `None` if the input doesn't match a known slash command.
pub fn parse_slash_command(input: &str) -> Option<(Intent, String)> {
    let trimmed = input.trim();

    for &(name, intent, requires_arg, accepts_trailing) in COMMAND_TABLE {
        // Exact match: `/done`
        if trimmed == name {
            return if requires_arg {
                None
            } else {
                Some((intent, String::new()))
            };
        }

        // Command with argument or optional note: `/now improving AIDX`, `/done shipped`
        if accepts_trailing {
            let prefix = format!("{name} ");
            if let Some(rest) = trimmed.strip_prefix(&prefix) {
                let arg = rest.trim().to_string();
                if requires_arg && arg.is_empty() {
                    return None;
                }
                return Some((intent, arg));
            }
        }
    }

    // Heuristic: questions end with ?
    if trimmed.ends_with('?') {
        return Some((Intent::QueryCurrent, trimmed.to_string()));
    }

    // Heuristic: bare "back" or "back to ..."
    let lower = trimmed.to_lowercase();
    if lower == "back" || lower.starts_with("back to ") {
        return Some((Intent::ReturnToParent, String::new()));
    }

    // No match — caller should treat as a note
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Title normalisation tests --

    #[test]
    fn normalise_strips_im() {
        assert_eq!(normalise_title("I'm improving AIDX"), "improving AIDX");
    }

    #[test]
    fn normalise_strips_i_am() {
        assert_eq!(normalise_title("I am debugging sync"), "debugging sync");
    }

    #[test]
    fn normalise_strips_working_on() {
        assert_eq!(normalise_title("working on docs"), "docs");
    }

    #[test]
    fn normalise_strips_im_working_on() {
        assert_eq!(
            normalise_title("I'm working on the component library"),
            "the component library"
        );
    }

    #[test]
    fn normalise_trims_whitespace() {
        assert_eq!(normalise_title("  improving AIDX  "), "improving AIDX");
    }

    #[test]
    fn normalise_preserves_acronyms() {
        assert_eq!(normalise_title("AIDX"), "AIDX");
    }

    #[test]
    fn normalise_case_insensitive_prefix() {
        assert_eq!(normalise_title("i'm improving AIDX"), "improving AIDX");
    }

    #[test]
    fn normalise_empty_after_strip_returns_original() {
        assert_eq!(normalise_title("I'm "), "I'm");
    }

    // -- Slash command parsing tests --

    #[test]
    fn parse_now_command() {
        let result = parse_slash_command("/now improving AIDX");
        assert_eq!(
            result,
            Some((Intent::SetCurrentThread, "improving AIDX".into()))
        );
    }

    #[test]
    fn parse_branch_command() {
        let result = parse_slash_command("/branch answering support");
        assert_eq!(
            result,
            Some((Intent::StartBranch, "answering support".into()))
        );
    }

    #[test]
    fn parse_back_command() {
        let result = parse_slash_command("/back");
        assert_eq!(result, Some((Intent::ReturnToParent, String::new())));
    }

    #[test]
    fn parse_note_command() {
        let result = parse_slash_command("/note article may help");
        assert_eq!(result, Some((Intent::AddNote, "article may help".into())));
    }

    #[test]
    fn parse_where_command() {
        let result = parse_slash_command("/where");
        assert_eq!(result, Some((Intent::QueryCurrent, String::new())));
    }

    #[test]
    fn parse_pause_command() {
        let result = parse_slash_command("/pause");
        assert_eq!(result, Some((Intent::Pause, String::new())));
    }

    #[test]
    fn parse_resume_command() {
        let result = parse_slash_command("/resume");
        assert_eq!(result, Some((Intent::Resume, String::new())));
    }

    #[test]
    fn parse_park_command() {
        let result = parse_slash_command("/park");
        assert_eq!(result, Some((Intent::Park, String::new())));
    }

    #[test]
    fn parse_done_command() {
        let result = parse_slash_command("/done");
        assert_eq!(result, Some((Intent::Done, String::new())));
    }

    #[test]
    fn parse_archive_command() {
        let result = parse_slash_command("/archive");
        assert_eq!(result, Some((Intent::Archive, String::new())));
    }

    #[test]
    fn parse_back_command_with_note() {
        let result = parse_slash_command("/back need more data first");
        assert_eq!(
            result,
            Some((Intent::ReturnToParent, "need more data first".into()))
        );
    }

    #[test]
    fn parse_pause_command_with_note() {
        let result = parse_slash_command("/pause blocked on review");
        assert_eq!(result, Some((Intent::Pause, "blocked on review".into())));
    }

    #[test]
    fn parse_resume_command_with_note() {
        let result = parse_slash_command("/resume revisit this tomorrow");
        assert_eq!(
            result,
            Some((Intent::Resume, "revisit this tomorrow".into()))
        );
    }

    #[test]
    fn parse_park_command_with_note() {
        let result = parse_slash_command("/park waiting on feedback");
        assert_eq!(result, Some((Intent::Park, "waiting on feedback".into())));
    }

    #[test]
    fn parse_done_command_with_note() {
        let result = parse_slash_command("/done shipped first pass");
        assert_eq!(result, Some((Intent::Done, "shipped first pass".into())));
    }

    #[test]
    fn parse_archive_command_with_note() {
        let result = parse_slash_command("/archive no longer needed");
        assert_eq!(result, Some((Intent::Archive, "no longer needed".into())));
    }

    #[test]
    fn parse_question_heuristic() {
        let result = parse_slash_command("what am I working on?");
        assert_eq!(
            result,
            Some((Intent::QueryCurrent, "what am I working on?".into()))
        );
    }

    #[test]
    fn parse_back_heuristic() {
        assert_eq!(
            parse_slash_command("back"),
            Some((Intent::ReturnToParent, String::new()))
        );
        assert_eq!(
            parse_slash_command("back to AIDX"),
            Some((Intent::ReturnToParent, String::new()))
        );
    }

    #[test]
    fn parse_where_rejects_trailing_text() {
        // `/where` is exact-match only — trailing text should not be silently dropped
        assert_eq!(parse_slash_command("/where anything"), None);
        // Note: `/where status?` still matches the `?` question heuristic,
        // which is correct — it becomes a QueryCurrent with the full text.
    }

    #[test]
    fn parse_plain_text_returns_none() {
        assert_eq!(parse_slash_command("reading article"), None);
    }

    #[test]
    fn parse_empty_input_returns_none() {
        assert_eq!(parse_slash_command(""), None);
    }
}
