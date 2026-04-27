use rig::{
    agent::Agent,
    client::{CompletionClient, ProviderClient},
    tool::ToolDyn,
};

pub fn get_bedrock_agent(
    model_name: String,
    preamble: Option<String>,
    tools: Vec<Box<dyn ToolDyn>>,
) -> Agent<rig_bedrock::completion::CompletionModel> {
    // There's no config provider because bedrock is configured solely by env. Following are some
    // environment that you can override to provide the necessary configuration to bedrock (apart
    // from the standard env like AWS_REGION)
    // - AWS_ENDPOINT_URL_BEDROCK_RUNTIME
    // - AWS_BEARER_TOKEN_BEDROCK
    let client = rig_bedrock::client::Client::from_env();
    let mut agent = client.agent(model_name);
    if let Some(p) = preamble {
        agent = agent.preamble(&p);
    }
    let agent = agent.tools(tools).build();

    agent
}
