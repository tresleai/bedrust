use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Deserialize, Serialize)]
pub struct BedrustConfig {
    pub aws_profile: String,
    pub supported_images: Vec<String>,
    pub caption_prompt: String,
    pub default_model: Option<ArgModels>,
    // FIX: Implement a better way for configuration defaults
    // for now if there is no configuration line use true
    #[serde(default = "_default_true")]
    pub show_banner: bool,
    pub inference_params: InferenceParams,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InferenceParams {
    pub temperature: f32,
    pub max_tokens: i32,
    pub top_p: f32,
}

#[derive(Clone, Serialize, Deserialize, Debug, Copy)]
pub enum ArgModels {
    Llama270b,
    Llama31405bInstruct,
    Llama3170bInstruct,
    Llama318bInstruct,
    CohereCommand,
    ClaudeV2,
    ClaudeV21,
    ClaudeV3Opus,
    ClaudeV3Sonnet,
    ClaudeV3Haiku,
    ClaudeV35Sonnet,
    ClaudeV352Sonnet,
    ClaudeV35Haiku,
    Jurrasic2Ultra,
    TitanTextExpressV1,
    Mixtral8x7bInstruct,
    Mistral7bInstruct,
    MistralLarge,
    MistralLarge2,
    NovaMicro,
    NovaLite,
    NovaPro,
}

impl Display for ArgModels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl ArgModels {
    pub fn to_str(&self) -> &'static str {
        match self {
            ArgModels::ClaudeV2 => "anthropic.claude-v2",
            ArgModels::ClaudeV21 => "anthropic.claude-v2:1",
            ArgModels::ClaudeV3Haiku => "anthropic.claude-3-haiku-20240307-v1:0",
            ArgModels::ClaudeV35Haiku => "anthropic.claude-3-5-haiku-20241022-v1:0",
            ArgModels::ClaudeV3Sonnet => "anthropic.claude-3-sonnet-20240229-v1:0",
            ArgModels::ClaudeV3Opus => "anthropic.claude-3-opus-20240229-v1:0",
            ArgModels::ClaudeV35Sonnet => "anthropic.claude-3-5-sonnet-20240620-v1:0",
            ArgModels::ClaudeV352Sonnet => "anthropic.claude-3-5-sonnet-20241022-v2:0",
            ArgModels::Llama270b => "meta.llama2-70b-chat-v1",
            ArgModels::Llama31405bInstruct => "meta.llama3-1-405b-instruct-v1:0",
            ArgModels::Llama3170bInstruct => "meta.llama3-1-70b-instruct-v1:0",
            ArgModels::Llama318bInstruct => "meta.llama3-1-8b-instruct-v1:0",
            ArgModels::CohereCommand => "cohere.command-text-v14",
            ArgModels::Jurrasic2Ultra => "ai21.j2-ultra-v1",
            ArgModels::TitanTextExpressV1 => "amazon.titan-text-express-v1",
            ArgModels::Mixtral8x7bInstruct => "mistral.mixtral-8x7b-instruct-v0:1",
            ArgModels::Mistral7bInstruct => "mistral.mistral-7b-instruct-v0:2",
            ArgModels::MistralLarge => "mistral.mistral-large-2402-v1:0",
            ArgModels::MistralLarge2 => "mistral.mistral-large-2407-v1:0",
            ArgModels::NovaMicro => "us.amazon.nova-micro-v1:0",
            ArgModels::NovaLite => "us.amazon.nova-lite-v1:0",
            ArgModels::NovaPro => "us.amazon.nova-pro-v1:0",
        }
    }
}
// ######################################## END ARGUMENT PARSING
// ######################################## CONST FUNCTIONS
// Used to set default values to struct fields during serialization
const fn _default_true() -> bool {
    true
}
// ######################################## END CONST FUNCTIONS
