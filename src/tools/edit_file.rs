use crate::utils::GLOBAL_EXECUTION_HANDLER;
use regex::RegexBuilder;
use rig::completion::ToolDefinition;
use rig::tool::{Tool, ToolError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;

/// Returns the 1-based line number at the given byte offset within `content`.
fn line_at(content: &str, byte_offset: usize) -> usize {
    content[..byte_offset].lines().count() + 1
}

#[derive(Deserialize)]
pub struct EditFileArgs {
    pub filepath: String,
    pub search: String,
    pub replace: String,
    pub replace_mode: Option<String>,
    pub search_mode: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct EditFile;

impl Tool for EditFile {
    const NAME: &'static str = "edit_file";
    type Error = ToolError;
    type Args = EditFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "edit_file".to_string(),
            description: "Find → replace. 'one' errors if >1 match.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "filepath": { "type": "string", "description": "File path" },
                    "search": { "type": "string", "description": "Search text or regex (see search_mode)" },
                    "replace": { "type": "string", "description": "Replacement text" },
                    "replace_mode": {
                        "type": "string",
                        "enum": ["one", "all"],
                        "description": "one = first match (error if >1). all = every match"
                    },
                    "search_mode": {
                        "type": "string",
                        "enum": ["literal", "regex"],
                        "description": "literal = exact match (default). regex = pattern, dot matches \\n"
                    }
                },
                "required": ["filepath", "search", "replace"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let replace_mode = args.replace_mode.unwrap_or_else(|| "one".to_string());
        let search_mode = args.search_mode.unwrap_or_else(|| "literal".to_string());
        let path = Path::new(&args.filepath);

        if !["one", "all"].contains(&replace_mode.as_str()) {
            return Err(ToolError::ToolCallError(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Bad replace_mode '{}'. Use 'one' or 'all'", replace_mode),
            ))));
        }

        if !["literal", "regex"].contains(&search_mode.as_str()) {
            return Err(ToolError::ToolCallError(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Bad search_mode '{}'. Use 'literal' or 'regex'",
                    search_mode
                ),
            ))));
        }

        let content = fs::read_to_string(path).map_err(|e| {
            ToolError::ToolCallError(Box::new(std::io::Error::new(
                e.kind(),
                format!("Read fail '{}': {}", args.filepath, e),
            )))
        })?;

        let (new_content, edits) = if search_mode == "regex" {
            let re = RegexBuilder::new(&args.search)
                .dot_matches_new_line(true)
                .build()
                .map_err(|e| {
                    ToolError::ToolCallError(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Bad regex '{}': {}", args.search, e),
                    )))
                })?;

            let matches: Vec<_> = re.find_iter(&content).collect();
            let match_count = matches.len();

            if match_count == 0 {
                return Err(ToolError::ToolCallError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("No match in '{}'", args.filepath),
                ))));
            }

            if replace_mode == "one" && match_count > 1 {
                return Err(ToolError::ToolCallError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("{} matches. Use 'all' or narrow search", match_count),
                ))));
            }

            let edits: Vec<serde_json::Value> = matches
                .iter()
                .map(|m| {
                    json!({
                        "line": line_at(&content, m.start()),
                        "text_replaced": m.as_str(),
                    })
                })
                .collect();

            let result = match replace_mode.as_str() {
                "one" => re
                    .replace(&content, regex::NoExpand(&args.replace))
                    .to_string(),
                _ => re
                    .replace_all(&content, regex::NoExpand(&args.replace))
                    .to_string(),
            };

            (result, edits)
        } else {
            let matches: Vec<_> = content.match_indices(&args.search).collect();
            let match_count = matches.len();

            if match_count == 0 {
                return Err(ToolError::ToolCallError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("No match in '{}'", args.filepath),
                ))));
            }

            if replace_mode == "one" && match_count > 1 {
                return Err(ToolError::ToolCallError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("{} matches. Use 'all' or narrow search", match_count),
                ))));
            }

            let edits: Vec<serde_json::Value> = matches
                .iter()
                .map(|(offset, _)| {
                    json!({
                        "line": line_at(&content, *offset),
                    })
                })
                .collect();

            let new_content = match replace_mode.as_str() {
                "one" => content.replacen(&args.search, &args.replace, 1),
                _ => content.replace(&args.search, &args.replace),
            };

            (new_content, edits)
        };

        fs::write(path, &new_content).map_err(|e| {
            ToolError::ToolCallError(Box::new(std::io::Error::new(
                e.kind(),
                format!("Write fail '{}': {}", args.filepath, e),
            )))
        })?;

        let _ = GLOBAL_EXECUTION_HANDLER.execute_on_main_thread("vim.cmd('checktime')");

        Ok(json!({
            "edits": edits,
            "count": edits.len(),
        })
        .to_string())
    }
}
