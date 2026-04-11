use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use rig::{
    agent::Agent,
    client::{CompletionClient, Nothing},
    completion::CompletionModel,
    providers::ollama,
    tool::{Tool, ToolDyn},
};

#[allow(dead_code)]
pub enum SupportedModels {
    Ollama {
        config: OllamaProviderConfig,
        model_name: String,
    },
}

pub fn get_agent(
    model: SupportedModels,
    preamble: Option<String>,
    tools: Vec<impl Tool + 'static>,
) -> Agent<impl CompletionModel> {
    match model {
        SupportedModels::Ollama { config, model_name } => {
            get_ollama_agent(config, model_name, preamble, tools)
        }
    }
}

pub struct OllamaProviderConfig {
    pub base_url: String,
    pub bearer: Option<String>,
}

impl Default for OllamaProviderConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:11434".to_string(),
            bearer: std::env::var("OLLAMA_API_KEY").ok(),
        }
    }
}

fn get_ollama_agent(
    config: OllamaProviderConfig,
    model_name: String,
    preamble: Option<String>,
    tools: Vec<impl Tool + 'static>,
) -> Agent<impl CompletionModel> {
    let mut headers = HeaderMap::new();
    if let Some(bearer) = config.bearer {
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", bearer)).unwrap(),
        );
    }
    let client = ollama::Client::builder()
        .base_url(config.base_url)
        .http_headers(headers)
        .api_key(Nothing)
        .build()
        .unwrap();
    let mut agent = client.agent(model_name);
    if let Some(p) = preamble {
        agent = agent.preamble(&p);
    }
    let agent = agent
        .tools(
            tools
                .into_iter()
                .map(|t| Box::new(t) as Box<dyn ToolDyn>)
                .collect(),
        )
        .build();

    agent
}
