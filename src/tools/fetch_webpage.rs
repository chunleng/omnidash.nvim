use dom_smoothie::{Config, Readability, TextMode};
use rig::completion::ToolDefinition;
use rig::tool::{Tool, ToolError};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
pub struct FetchWebpageArgs {
    pub url: String,
    pub include_links: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct FetchWebpage;

impl Tool for FetchWebpage {
    const NAME: &'static str = "fetch_webpage";
    type Error = ToolError;
    type Args = FetchWebpageArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "fetch_webpage".to_string(),
            description: "Fetch webpage → readable text for LLM".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to fetch"
                    },
                    "include_links": {
                        "type": "boolean",
                        "description": "Keep hyperlinks. Default: true",
                        "default": true
                    }
                },
                "required": ["url"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let include_links = args.include_links.unwrap_or(true);

        let html = reqwest::get(&args.url)
            .await
            .map_err(|e| {
                ToolError::ToolCallError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to fetch URL '{}': {}", args.url, e),
                )))
            })?
            .text()
            .await
            .map_err(|e| {
                ToolError::ToolCallError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read response body: {}", e),
                )))
            })?;

        let text_mode = if include_links {
            TextMode::Markdown
        } else {
            TextMode::Formatted
        };

        let config = Config {
            text_mode,
            ..Default::default()
        };

        let mut readability =
            Readability::new(html, Some(&args.url), Some(config)).map_err(|e| {
                ToolError::ToolCallError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to initialize Readability: {}", e),
                )))
            })?;

        let article = readability.parse().map_err(|e| {
            ToolError::ToolCallError(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to parse article: {}", e),
            )))
        })?;

        Ok(format!(
            "Title: {}\n\nContent:\n{}",
            article.title, article.text_content
        ))
    }
}
