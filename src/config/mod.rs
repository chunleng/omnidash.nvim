use std::collections::HashMap;

use crate::{
    chat::TenonAgent,
    clients::{OllamaProviderConfig, ProviderConfig, SupportedModels},
};

pub mod user;

#[derive(Debug, Clone)]
pub struct TenonConfig {
    pub connectors: HashMap<String, ProviderConfig>,
    pub agents: HashMap<String, TenonAgent>,
    default_agent: String,
}

impl TenonConfig {
    pub fn default_agent(&self) -> TenonAgent {
        self.agents
            .get(&self.default_agent)
            .expect("the program failed to enforce default_agent validation")
            .clone()
    }
}

impl Default for TenonConfig {
    fn default() -> Self {
        let ollama_cloud_provider = ProviderConfig::Ollama(OllamaProviderConfig {
            base_url: "https://ollama.com".to_string(),
            ..Default::default()
        });
        let mut default_providers: HashMap<String, ProviderConfig> = HashMap::new();
        default_providers.insert("ollama_cloud".to_string(), ollama_cloud_provider.clone());
        let mut default_agents: HashMap<String, TenonAgent> = HashMap::new();
        let default_agent_name = "default".to_string();
        default_agents.insert(
            default_agent_name.clone(),
            TenonAgent::new(
                SupportedModels {
                    config: ollama_cloud_provider,
                    model_name: "glm-5.1".to_string(),
                },
                None,
                &[
                    "create_file",
                    "edit_file",
                    "fetch_webpage",
                    "list_file",
                    "read_file",
                    "search_text",
                    "web_search",
                    "think",
                ],
            ),
        );
        TenonConfig {
            connectors: default_providers,
            agents: default_agents,
            default_agent: default_agent_name,
        }
    }
}
