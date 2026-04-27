use rig::{agent::Agent, client::CompletionClient, providers::gemini, tool::ToolDyn};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GeminiProviderConfig {
    pub base_url: String,
    pub api_key: String,
}

impl Default for GeminiProviderConfig {
    fn default() -> Self {
        Self {
            base_url: "https://generativelanguage.googleapis.com".to_string(),
            api_key: std::env::var("GEMINI_API_KEY").unwrap_or_default(),
        }
    }
}

pub fn get_gemini_agent(
    config: GeminiProviderConfig,
    model_name: String,
    preamble: Option<String>,
    tools: Vec<Box<dyn ToolDyn>>,
) -> Agent<gemini::CompletionModel> {
    let client = gemini::Client::builder()
        .base_url(config.base_url)
        .api_key(config.api_key)
        .build()
        .unwrap();
    let mut agent = client.agent(model_name);
    if let Some(p) = preamble {
        agent = agent.preamble(&p);
    }
    let agent = agent.tools(tools).build();

    agent
}
