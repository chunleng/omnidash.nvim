use rig::{
    OneOrMany,
    message::{AssistantContent, Image, Message, ToolResult, ToolResultContent, UserContent},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenonUserTextMessage(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TenonUserMessage {
    Text(TenonUserTextMessage),
}

impl From<TenonUserMessage> for Message {
    fn from(value: TenonUserMessage) -> Self {
        match value {
            TenonUserMessage::Text(TenonUserTextMessage(msg)) => Message::User {
                content: OneOrMany::one(UserContent::text(msg)),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TenonAssistantMessageContent {
    Text(String),
}

impl From<TenonAssistantMessageContent> for AssistantContent {
    fn from(value: TenonAssistantMessageContent) -> Self {
        match value {
            TenonAssistantMessageContent::Text(s) => AssistantContent::text(s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenonAssistantMessage {
    pub reasoning: Option<String>,
    pub content: Vec<TenonAssistantMessageContent>,
}

impl From<TenonAssistantMessage> for Option<Message> {
    fn from(value: TenonAssistantMessage) -> Self {
        // reasoning is not return to consciously reduce context
        Some(Message::Assistant {
            id: None,
            content: OneOrMany::many(value.content.into_iter().map(|x| x.into())).ok()?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenonToolCall {
    pub id: String,
    pub internal_call_id: String,
    pub name: String,
    pub args: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TenonToolResult {
    Text(rig::agent::Text),
    Image(Image),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenonToolError(pub String);

impl TenonToolError {
    /// Strip rig's internal wrapping prefixes for display.
    /// E.g. "Toolset error: ToolCallError: ToolCallError: read_file ..."
    ///   → "read_file ..."
    pub fn display_message(&self) -> &str {
        let mut s = self.0.strip_prefix("Toolset error: ").unwrap_or(&self.0);
        while let Some(stripped) = s.strip_prefix("ToolCallError: ") {
            s = stripped;
        }
        s
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenonToolLog {
    pub tool_call: TenonToolCall,
    pub tool_result: Option<Result<TenonToolResult, TenonToolError>>,
}

impl From<TenonToolLog> for Vec<Message> {
    fn from(value: TenonToolLog) -> Self {
        let mut messages = vec![Message::Assistant {
            id: None,
            content: OneOrMany::one(AssistantContent::tool_call(
                value.tool_call.id.clone(),
                value.tool_call.name,
                value.tool_call.args,
            )),
        }];
        if let Some(res) = value.tool_result {
            let tool_result_content = match &res {
                Ok(TenonToolResult::Text(text)) => {
                    OneOrMany::one(ToolResultContent::Text(text.clone()))
                }
                Ok(TenonToolResult::Image(img)) => {
                    OneOrMany::one(ToolResultContent::Image(img.clone()))
                }
                Err(err) => OneOrMany::one(ToolResultContent::text(&err.0)),
            };
            messages.push(Message::User {
                content: OneOrMany::one(UserContent::ToolResult(ToolResult {
                    id: value.tool_call.id,
                    call_id: None,
                    content: tool_result_content,
                })),
            });
        }

        messages
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TenonLog {
    User(TenonUserMessage),
    Assistant(TenonAssistantMessage),
    Tool(TenonToolLog),
}

impl From<TenonLog> for Vec<Message> {
    fn from(value: TenonLog) -> Self {
        match value {
            TenonLog::User(user_message) => vec![user_message.into()],
            TenonLog::Assistant(assistant_message) => {
                match Option::<Message>::from(assistant_message) {
                    Some(x) => vec![x.into()],
                    None => vec![],
                }
            }
            TenonLog::Tool(tool_log) => tool_log.into(),
        }
    }
}
