use crate::get_application_config;
use rig::completion::ToolDefinition;
use rig::tool::{Tool, ToolError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

/// Shell metacharacters that indicate shell features (pipes, redirects, etc.)
const SHELL_METACHARACTERS: &[&str] = &["|", "&&", ";", ">", "<", "$("];

/// Hard cap on combined stdout+stderr output size (bytes).
const OUTPUT_CAP: usize = 64 * 1024;

#[derive(Deserialize)]
pub struct RunArgs {
    pub command: String,
    pub cwd: Option<String>,
    pub timeout: Option<u64>,
    pub filter: Option<String>,
    pub direction: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Run;

#[derive(Serialize)]
struct RunOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
    truncated: bool,
}

/// Check a command string for shell metacharacters.
/// Returns the first metacharacter found, or None if clean.
fn find_shell_metacharacter(command: &str) -> Option<&'static str> {
    for &mc in SHELL_METACHARACTERS {
        if command.contains(mc) {
            return Some(mc);
        }
    }
    None
}

/// Arg allowance for a whitelist pattern.
enum ArgAllowance {
    /// No additional arguments beyond pattern tokens.
    Exact,
    /// Exactly one additional argument allowed.
    OneArg,
    /// Any number of additional arguments allowed.
    AnyArgs,
}

/// Parse a whitelist pattern into (command_tokens, arg_allowance).
///
/// - `"make"`       → `(["make"], Exact)`   — exact, no args
/// - `"make ?"`    → `(["make"], OneArg)`   — one arg only
/// - `"make *"`    → `(["make"], AnyArgs)`  — any number of args
/// - `"git log"`   → `(["git", "log"], Exact)` — exact subcommand
/// - `"git log *"` → `(["git", "log"], AnyArgs)` — subcommand with any args
fn parse_whitelist_pattern(pattern: &str) -> (Vec<String>, ArgAllowance) {
    let trimmed = pattern.trim();
    let tokens: Vec<String> = shlex::split(trimmed).unwrap_or_default();

    let allowance = if trimmed.ends_with(" *") {
        ArgAllowance::AnyArgs
    } else if trimmed.ends_with(" ?") {
        ArgAllowance::OneArg
    } else {
        ArgAllowance::Exact
    };

    let mut cmd_tokens = tokens;
    // Strip the trailing wildcard token (* or ?) if present
    match cmd_tokens.last().map(|t| t.as_str()) {
        Some("*") | Some("?") => {
            cmd_tokens.pop();
        }
        _ => {}
    }

    (cmd_tokens, allowance)
}

/// Check if a parsed command matches any whitelist pattern.
fn command_matches_whitelist(command_tokens: &[String], whitelist: &[String]) -> bool {
    for pattern in whitelist {
        let (pattern_tokens, allowance) = parse_whitelist_pattern(pattern);

        // Command must have at least as many tokens as the pattern
        if command_tokens.len() < pattern_tokens.len() {
            continue;
        }

        // All leading tokens must match exactly
        let leading_match = command_tokens
            .iter()
            .zip(pattern_tokens.iter())
            .all(|(a, b)| a == b);

        if !leading_match {
            continue;
        }

        let extra_args = command_tokens.len() - pattern_tokens.len();

        let matches = match allowance {
            ArgAllowance::Exact => extra_args == 0,
            ArgAllowance::OneArg => extra_args == 1,
            ArgAllowance::AnyArgs => true,
        };

        if matches {
            return true;
        }
    }
    false
}

/// Apply filter, direction, and limit to stdout lines.
fn apply_output_filters(
    stdout: &str,
    filter: Option<&str>,
    direction: Option<&str>,
    limit: Option<usize>,
) -> String {
    let mut lines: Vec<&str> = stdout.lines().collect();

    if let Some(f) = filter {
        lines.retain(|line| line.contains(f));
    }

    if let Some(dir) = direction {
        if dir == "head" {
            // keep from the start
        } else {
            // "tail" (default) — keep from the end
            lines.reverse();
        }
    }

    if let Some(n) = limit {
        lines.truncate(n);
    }

    // If we reversed for tail, reverse back
    if let Some(dir) = direction {
        if dir != "head" {
            lines.reverse();
        }
    }

    lines.join("\n")
}

impl Tool for Run {
    const NAME: &'static str = "run";
    type Error = ToolError;
    type Args = RunArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        let config = get_application_config();
        let allowed_list = &config.tools.run.whitelist;
        let allowed_desc = if allowed_list.is_empty() {
            "(none configured)".to_string()
        } else {
            allowed_list.join(", ")
        };

        ToolDefinition {
            name: "run".to_string(),
            description: format!(
                "Execute a permitted command.\n\nAllowed: {}\n\nNo shell features (pipes, &&, redirects, $()). Use filter/limit/direction to reduce output instead of piping.\n\nOutput: stdout (filtered by limit) + all stderr.",
                allowed_desc
            ),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Command to execute. Quotes supported."
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Working directory. Default: cwd."
                    },
                    "timeout": {
                        "type": "integer",
                        "description": "Seconds before kill. Default: 30."
                    },
                    "filter": {
                        "type": "string",
                        "description": "Only stdout lines containing this substring."
                    },
                    "direction": {
                        "type": "string",
                        "description": "\"head\"|\"tail\". Which end to keep. Default: tail."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max stdout lines. null = no limit."
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Check for shell metacharacters
        if let Some(mc) = find_shell_metacharacter(&args.command) {
            return Err(ToolError::ToolCallError(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Shell metacharacter '{}' not allowed. The run tool executes commands directly — no shell features (pipes, &&, redirects, $()). Use filter/limit/direction to reduce output.",
                    mc
                ),
            ))));
        }

        // Parse the command
        let command_tokens = shlex::split(&args.command).ok_or_else(|| {
            ToolError::ToolCallError(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Failed to parse command: '{}'", args.command),
            )))
        })?;

        if command_tokens.is_empty() {
            return Err(ToolError::ToolCallError(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Empty command".to_string(),
            ))));
        }

        // Check whitelist
        let config = get_application_config();
        let whitelist = &config.tools.run.whitelist;

        if !command_matches_whitelist(&command_tokens, whitelist) {
            return Err(ToolError::ToolCallError(Box::new(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                if whitelist.is_empty() {
                    "No commands allowed — whitelist is empty. Configure run.whitelist in setup()."
                        .to_string()
                } else {
                    format!(
                        "Command '{}' not allowed. Allowed patterns: {}",
                        args.command,
                        whitelist.join(", ")
                    )
                },
            ))));
        }

        // Build the process
        let program = &command_tokens[0];
        let program_args = &command_tokens[1..];

        let timeout_secs = args.timeout.unwrap_or(30);

        let mut cmd = Command::new(program);
        cmd.args(program_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(ref cwd) = args.cwd {
            cmd.current_dir(cwd);
        }

        let child = cmd.spawn().map_err(|e| {
            ToolError::ToolCallError(Box::new(std::io::Error::new(
                e.kind(),
                format!("Failed to spawn '{}': {}", program, e),
            )))
        })?;

        // Run with timeout
        let result =
            tokio::time::timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await;

        let output = match result {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => {
                return Err(ToolError::ToolCallError(Box::new(std::io::Error::new(
                    e.kind(),
                    format!("Process error: {}", e),
                ))));
            }
            Err(_) => {
                // Timeout — try to get partial output by killing
                return Err(ToolError::ToolCallError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!(
                        "Command timed out after {}s: '{}'",
                        timeout_secs, args.command
                    ),
                ))));
            }
        };

        let exit_code = output.status.code().unwrap_or(-1);
        let raw_stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let raw_stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Apply output filters to stdout only
        let filtered_stdout = apply_output_filters(
            &raw_stdout,
            args.filter.as_deref(),
            args.direction.as_deref(),
            args.limit,
        );

        // Check truncation on combined filtered output + stderr
        let combined_len = filtered_stdout.len() + raw_stderr.len();
        let truncated = combined_len > OUTPUT_CAP;

        let result = RunOutput {
            exit_code,
            stdout: filtered_stdout,
            stderr: raw_stderr,
            truncated,
        };

        Ok(serde_json::to_string(&result).unwrap_or_else(|_| {
            r#"{"exit_code":-1,"stdout":"","stderr":"","truncated":false}"#.to_string()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_whitelist_pattern_exact() {
        let (tokens, allowance) = parse_whitelist_pattern("git status");
        assert_eq!(tokens, vec!["git", "status"]);
        assert!(matches!(allowance, ArgAllowance::Exact));
    }

    #[test]
    fn test_parse_whitelist_pattern_wildcard() {
        let (tokens, allowance) = parse_whitelist_pattern("git log *");
        assert_eq!(tokens, vec!["git", "log"]);
        assert!(matches!(allowance, ArgAllowance::AnyArgs));
    }

    #[test]
    fn test_parse_whitelist_pattern_single_arg() {
        let (tokens, allowance) = parse_whitelist_pattern("make ?");
        assert_eq!(tokens, vec!["make"]);
        assert!(matches!(allowance, ArgAllowance::OneArg));
    }

    #[test]
    fn test_parse_whitelist_pattern_single_command() {
        let (tokens, allowance) = parse_whitelist_pattern("make");
        assert_eq!(tokens, vec!["make"]);
        assert!(matches!(allowance, ArgAllowance::Exact));
    }

    #[test]
    fn test_command_matches_exact() {
        let whitelist = vec!["git status".to_string()];
        assert!(command_matches_whitelist(
            &vec!["git".to_string(), "status".to_string()],
            &whitelist
        ));
        // Extra args should NOT match exact pattern
        assert!(!command_matches_whitelist(
            &vec![
                "git".to_string(),
                "status".to_string(),
                "--short".to_string()
            ],
            &whitelist
        ));
    }

    #[test]
    fn test_command_matches_wildcard() {
        let whitelist = vec!["git log *".to_string()];
        assert!(command_matches_whitelist(
            &vec![
                "git".to_string(),
                "log".to_string(),
                "--oneline".to_string()
            ],
            &whitelist
        ));
        // Zero extra args still matches * pattern
        assert!(command_matches_whitelist(
            &vec!["git".to_string(), "log".to_string()],
            &whitelist
        ));
        assert!(!command_matches_whitelist(
            &vec!["git".to_string(), "diff".to_string()],
            &whitelist
        ));
    }

    #[test]
    fn test_command_matches_one_arg() {
        let whitelist = vec!["make ?".to_string()];
        // Exactly one extra arg → match
        assert!(command_matches_whitelist(
            &vec!["make".to_string(), "build".to_string()],
            &whitelist
        ));
        // No extra args → no match (pattern requires one arg)
        assert!(!command_matches_whitelist(
            &vec!["make".to_string()],
            &whitelist
        ));
        // Two extra args → no match
        assert!(!command_matches_whitelist(
            &vec!["make".to_string(), "build".to_string(), "-j4".to_string()],
            &whitelist
        ));
    }

    #[test]
    fn test_command_matches_single_exact() {
        let whitelist = vec!["make".to_string()];
        assert!(command_matches_whitelist(
            &vec!["make".to_string()],
            &whitelist
        ));
        assert!(!command_matches_whitelist(
            &vec!["make".to_string(), "build".to_string()],
            &whitelist
        ));
    }

    #[test]
    fn test_find_shell_metacharacter() {
        assert_eq!(find_shell_metacharacter("ls | grep foo"), Some("|"));
        assert_eq!(find_shell_metacharacter("make && make test"), Some("&&"));
        assert_eq!(find_shell_metacharacter("echo $(pwd)"), Some("$("));
        assert_eq!(find_shell_metacharacter("echo hi > out"), Some(">"));
        assert_eq!(find_shell_metacharacter("cat < in"), Some("<"));
        assert_eq!(find_shell_metacharacter("make ; echo done"), Some(";"));
        assert_eq!(find_shell_metacharacter("cargo build"), None);
    }

    #[test]
    fn test_apply_output_filters_no_filters() {
        let stdout = "line1\nline2\nline3";
        let result = apply_output_filters(stdout, None, None, None);
        assert_eq!(result, "line1\nline2\nline3");
    }

    #[test]
    fn test_apply_output_filters_with_filter() {
        let stdout = "error: bad\ninfo: ok\nerror: worse";
        let result = apply_output_filters(stdout, Some("error"), None, None);
        assert_eq!(result, "error: bad\nerror: worse");
    }

    #[test]
    fn test_apply_output_filters_with_limit_tail() {
        let stdout = "line1\nline2\nline3\nline4\nline5";
        let result = apply_output_filters(stdout, None, Some("tail"), Some(2));
        assert_eq!(result, "line4\nline5");
    }

    #[test]
    fn test_apply_output_filters_with_limit_head() {
        let stdout = "line1\nline2\nline3\nline4\nline5";
        let result = apply_output_filters(stdout, None, Some("head"), Some(2));
        assert_eq!(result, "line1\nline2");
    }

    #[test]
    fn test_command_matches_empty_whitelist() {
        let whitelist: Vec<String> = vec![];
        assert!(!command_matches_whitelist(
            &vec!["git".to_string()],
            &whitelist
        ));
    }
}
