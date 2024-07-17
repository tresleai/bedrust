pub mod claude;
pub mod claudev3;
pub mod cohere;
pub mod jurrasic2;
pub mod llama2;
pub mod mistral;
pub mod titan;
use claude::{ClaudeV21Config, ClaudeV2Config};
use claudev3::ClaudeV3Config;
use cohere::CohereCommandConfig;
use jurrasic2::Jurrasic2UltraConfig;
use llama2::Llama270bConfig;
use mistral::{Mistral7bInstruct, MistralLarge, Mixtral8x7bInstruct};
use serde::{Deserialize, Serialize};
use titan::TitanTextExpressV1Config;

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelConfigs {
    pub llama270b: Llama270bConfig,
    pub cohere_command: CohereCommandConfig,
    pub claude_v2: ClaudeV2Config,
    pub claude_v21: ClaudeV21Config,
    pub claude_v3: ClaudeV3Config,
    pub jurrasic_2_ultra: Jurrasic2UltraConfig,
    pub titan_text_express_v1: TitanTextExpressV1Config,
    pub mixtral_8x7b_instruct: Mixtral8x7bInstruct,
    pub mistral_7b_instruct: Mistral7bInstruct,
    pub mistral_large: MistralLarge,
}
