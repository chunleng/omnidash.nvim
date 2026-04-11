use crate::{tools::ReadFile, utils::notify};
use futures::stream::StreamExt;
use nvim_oxi::api::types::LogLevel;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use rig::{
    OneOrMany,
    agent::MultiTurnStreamItem,
    client::{CompletionClient, Nothing},
    completion::{GetTokenUsage, Usage},
    message::{AssistantContent, Message, ToolCall, UserContent},
    providers::ollama,
    streaming::{StreamedAssistantContent, StreamedUserContent, StreamingChat},
};
use std::{
    collections::{HashMap, LinkedList},
    sync::{Arc, RwLock},
};

pub struct ChatProcess {
    pub logs: Arc<RwLock<LinkedList<Message>>>,
    pub usage: Arc<RwLock<Option<Usage>>>,
}

impl ChatProcess {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(LinkedList::new())),
            usage: Arc::new(RwLock::new(None)),
        }
    }

    pub fn send_message(&mut self, message: String) {
        if let Ok(mut logs) = self.logs.write() {
            logs.push_back(Message::User {
                content: OneOrMany::one(UserContent::text(message.clone())),
            });
        }

        let logs_clone = Arc::clone(&self.logs);
        let usage_clone = Arc::clone(&self.usage);
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut headers = HeaderMap::new();
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!(
                        "Bearer {}",
                        std::env::var("OLLAMA_API_KEY").expect("OLLAMA_API_KEY must be set")
                    ))
                    .unwrap(),
                );
                let client = ollama::Client::builder()
                    .base_url("https://ollama.com")
                    .http_headers(headers)
                    .api_key(Nothing)
                    .build()
                    .unwrap();
                let agent = client
                    // TODO https://github.com/ollama/ollama/issues/14567
                    // gemini-3-flash-preview tools does not work because it requires additional
                    // `thought_signature`
                    // .agent("gemini-3-flash-preview")
                    .agent("glm-5")
                    .tool(ReadFile)
                    .build();

                let chat_history;
                if let Ok(logs) = logs_clone.read() {
                    chat_history = logs.iter().cloned().collect::<Vec<_>>();
                } else {
                    todo!("fix after error is introduced")
                }

                let mut stream = agent.stream_chat(message, chat_history).multi_turn(3).await;
                let mut full_response = String::new();
                if let Ok(mut logs) = logs_clone.write() {
                    logs.push_back(Message::Assistant {
                        id: None,
                        content: OneOrMany::one(AssistantContent::text(full_response.clone())),
                    });
                }
                let mut tools_lookup: HashMap<String, ToolCall> = HashMap::new();
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(MultiTurnStreamItem::StreamUserItem(
                            StreamedUserContent::ToolResult {
                                tool_result,
                                internal_call_id,
                                ..
                            },
                        )) => {
                            if let Ok(mut logs) = logs_clone.write() {
                                logs.push_back(Message::User {
                                    content: OneOrMany::one(UserContent::tool_result_with_call_id(
                                        tool_result.id,
                                        internal_call_id,
                                        tool_result.content,
                                    )),
                                });
                                tools_lookup = HashMap::new();
                                full_response = "".to_string();
                                logs.push_back(Message::Assistant {
                                    id: None,
                                    content: OneOrMany::one(AssistantContent::text(
                                        full_response.clone(),
                                    )),
                                });
                            }
                        }
                        Ok(MultiTurnStreamItem::StreamAssistantItem(
                            StreamedAssistantContent::Text(text_struct),
                        )) => {
                            full_response.push_str(&text_struct.text);
                            if let Ok(mut logs) = logs_clone.write() {
                                // TODO make this more efficient
                                logs.pop_back();
                                let mut content =
                                    vec![AssistantContent::text(full_response.clone())];
                                content.extend(
                                    tools_lookup
                                        .values()
                                        .cloned()
                                        .map(|tc: ToolCall| AssistantContent::ToolCall(tc)),
                                );
                                logs.push_back(Message::Assistant {
                                    id: None,
                                    content: OneOrMany::many(content).unwrap(),
                                });
                            }
                        }
                        Ok(MultiTurnStreamItem::StreamAssistantItem(
                            StreamedAssistantContent::ToolCall {
                                tool_call,
                                internal_call_id,
                            },
                        )) => {
                            tools_lookup.insert(internal_call_id.clone(), tool_call.into());
                            if let Ok(mut logs) = logs_clone.write() {
                                logs.pop_back();
                                let mut content =
                                    vec![AssistantContent::text(full_response.clone())];
                                content.extend(tools_lookup.values().cloned().map(
                                    move |tc: ToolCall| {
                                        AssistantContent::tool_call(
                                            internal_call_id.clone(),
                                            tc.function.name,
                                            tc.function.arguments,
                                        )
                                    },
                                ));
                                logs.push_back(Message::Assistant {
                                    id: None,
                                    content: OneOrMany::many(content).unwrap(),
                                });
                            }
                        }
                        Ok(MultiTurnStreamItem::StreamAssistantItem(
                            StreamedAssistantContent::Final(final_response),
                        )) => {
                            if let Some(usage) = final_response.token_usage() {
                                if let Ok(mut usage_lock) = usage_clone.write() {
                                    *usage_lock = Some(usage);
                                }
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            notify(format!("{}", e), LogLevel::Error);
                            break;
                        }
                    }
                }
            });
        });
    }
}
