use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_types::region::Region;
use core::panic;
use std::{env, io};

use anyhow::Result;

use aws_sdk_bedrockruntime::Client;
use aws_sdk_bedrockruntime::primitives::Blob;
use aws_sdk_bedrockruntime::types::ResponseStream;

use serde_json::{Value};

use std::io::Write;

//========================================
pub struct BedrockCall {
    pub body: Blob,
    pub content_type: String,
    pub accept: String,
    pub model_id: String,
}

impl BedrockCall {
    pub fn new(body: Blob, content_type: String, accept: String, model_id: String ) -> BedrockCall {
        BedrockCall {
            body,
            content_type,
            accept,
            model_id,
        }
    }
}


pub enum BedrockCallSum {
    CohereBCS { model_id: String, body: CohereBody},
    ClaudeBCS { model_id: String, body: ClaudeBody},
    Llama2BCS { model_id: String, body: Llama2Body}    
}

// Use a sum type here, 
fn bcs_to_bedrock_call(bcs: BedrockCallSum) ->  Result<BedrockCall> {
    match bcs {
        BedrockCallSum::CohereBCS { model_id, body } => {
            Ok(BedrockCall::new(body.convert_to_blob()?, "application/json".into(), "*/*".into(), model_id))
        }
        BedrockCallSum::ClaudeBCS { model_id, body } => {
            Ok(BedrockCall::new(body.convert_to_blob()?, "application/json".into(), "*/*".into(), model_id))
        }
        BedrockCallSum::Llama2BCS { model_id, body } => {
            Ok(BedrockCall::new(body.convert_to_blob()?, "application/json".into(), "*/*".into(), model_id))
        }
	
    }
}


// Create a BedrockCallSum with sensible defaults for each model
fn q_to_bcs_with_defaults(question: String, model_id: &str) -> Result<BedrockCallSum> {
    match model_id {
        "meta.llama2-70b-chat-v1" => {
            let llama2_body = Llama2Body::new(
                // prompt
                question.to_string(),
                // temperature
                1.0,
                // p
                0.1,
                // max_gen_len
                1024
                );
	    Ok(BedrockCallSum::Llama2BCS{model_id: String::from("meta.llama2-70b-chat-v1"), body: llama2_body})
	    
        },
        "cohere.command-text-v14" => {
            let cohere_body = CohereBody::new(
                // prompt
                question.to_string(),
                // max tokens
                500,
                // temperature
                1.0,
                // p
                0.1,
                // k
                1,
                // stop sequences
                Vec::new(),
                // stream
                true,
                );

	    Ok(BedrockCallSum::CohereBCS{model_id: String::from("cohere.command-text-v14"), body: cohere_body})
        },
        "anthropic.claude-v2" => {
            let claude_body = ClaudeBody::new(
                // prompt
                format!("\n\nHuman: {}\n\nAssistant:", question).to_string(),
                // temp
                1.0,
                // top p
                1.0,
                // top k
                250,
                // max tokens to sample
                500,
                // stop sequences
                Vec::new(),
            );
	    Ok(BedrockCallSum::ClaudeBCS{model_id: String::from("anthropic.claude-v2"), body: claude_body})
	},
	&_ => todo!()
    }
}


pub fn mk_bedrock_call(question: String, model_id: &str) -> Result<BedrockCall> {
    let bcs = q_to_bcs_with_defaults(question.to_string(), &model_id)?;
    bcs_to_bedrock_call(bcs)
}

pub async fn ask_bedrock(question: String, model_id: &str, client: Client) -> Result<()>{ 
    let bcall = mk_bedrock_call(question, model_id)?;
    call_bedrock_stream(client, bcall).await;
    Ok(())
}

//######################################## COHERE

#[derive(serde::Serialize, Debug)]
pub struct CohereBody {
    pub prompt: String,
    pub max_tokens: i32,
    pub temperature: f32,
    pub p: f32,
    pub k: i32,
    pub stop_sequences: Vec<String>,
    pub stream: bool,
}

impl CohereBody {
    pub fn new(prompt: String, max_tokens: i32, temperature: f32, p: f32, k: i32, stop_sequences: Vec<String>, stream: bool) -> CohereBody {
        CohereBody {
            prompt,
            max_tokens,
            temperature,
            p,
            k,
            stop_sequences,
            stream,
        }
    }

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string);
        Ok(body)
    }
}

#[derive(serde::Deserialize)]
pub struct CohereResponseGenerations {
    generations: Vec<CohereResponseText>,
}
#[derive(serde::Deserialize, Debug)]
pub struct CohereResponseText {
    text: String,
}
//######################################## END COHERE

//######################################## CLAUDE
#[derive(serde::Serialize, Debug)]
pub struct ClaudeBody {
    pub prompt: String,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub max_tokens_to_sample: i32,
    pub stop_sequences: Vec<String>,
}

impl ClaudeBody {
    pub fn new(prompt: String, temperature: f32, top_p: f32, top_k: i32, max_tokens_to_sample: i32, stop_sequences: Vec<String>) -> ClaudeBody {
        ClaudeBody {
            prompt,
            temperature,
            top_p,
            top_k,
            max_tokens_to_sample,
            stop_sequences,
        }
    }

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string);
        Ok(body)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct ClaudeResponse {
    completion: String,
}
//######################################## END CLAUDE
//
//######################################## LLAMA2
#[derive(serde::Serialize, Debug)]
pub struct Llama2Body {
    pub prompt: String,
    pub temperature: f32,
    pub top_p: f32,
    pub max_gen_len: i32,
}

impl Llama2Body {
    pub fn new(prompt: String, temperature: f32, top_p: f32, max_gen_len: i32) -> Llama2Body {
        Llama2Body {
            prompt,
            temperature,
            top_p,
            max_gen_len,
        }
    }

    pub fn convert_to_blob(&self) -> Result<Blob> {
        let blob_string = serde_json::to_vec(&self)?;
        let body: Blob = Blob::new(blob_string);
        Ok(body)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Llama2Response {
    generation: String,
}
//######################################## END CLAUDE
//========================================


pub fn convert_json(s: &str) -> Result<String> {
    let v: Value = serde_json::from_str(s)?;
    // future_highway: we convert to Option<&str>
    // then using .ok_or() we get the &str
    // then to get the String we use .to_string()
    let response = v["generations"][0]["text"]
        .as_str()
        .ok_or(anyhow::anyhow!("Not really a string"))?
        .to_string();
    Ok(response)
}

pub async fn configure_aws(s: String) -> aws_config::SdkConfig {
    let provider = RegionProviderChain::first_try(env::var("AWS_DEFAULT_REGION").ok().map(Region::new))
        .or_default_provider()
        .or_else(Region::new(s));

    aws_config::defaults(BehaviorVersion::latest())
        .region(provider)
        .load()
        .await

}

pub async fn call_bedrock(bc: Client, c: BedrockCall) -> Result<String>{

    let response = bc.invoke_model()
    .body(c.body)
    .content_type(c.content_type)
    .accept(c.accept)
    .model_id(c.model_id)
    .send()
    .await?;

    let response_body = response
        .body
        .into_inner();

    let reponse_string = String::from_utf8(response_body)?;
    Ok(reponse_string)

}

pub async fn call_bedrock_stream(bc: Client, c: BedrockCall) -> Result<()>{

    let mut resp =  bc.invoke_model_with_response_stream()
        .body(c.body)
        .content_type(c.content_type)
        .model_id(&c.model_id)
        .send()
        .await?;

    let mut output = String::new();

    while let Some(event) = resp.body.recv().await? {
        match event {
            ResponseStream::Chunk(payload_part) => {
                if let Some(payload_bytes) = payload_part.bytes {
                    match c.model_id.as_str()  {
                        "meta.llama2-70b-chat-v1" => {
                            if let Ok(good_response_chunk) = serde_json::from_slice::<Llama2Response>(payload_bytes.as_ref()) {
                                print!("{}", good_response_chunk.generation);
                                io::stdout().flush().unwrap();
                                output += &good_response_chunk.generation;
                            }
                        },
                        "cohere.command-text-v14" => {
                            if let Ok(good_response_chunk) = serde_json::from_slice::<CohereResponseText>(payload_bytes.as_ref()) {
                                print!("{}", good_response_chunk.text);
                                io::stdout().flush().unwrap();
                                output += &good_response_chunk.text;
                            }
                        },
                        "anthropic.claude-v2" => {
                            if let Ok(good_response_chunk) = serde_json::from_slice::<ClaudeResponse>(payload_bytes.as_ref()) {
                                print!("{}", good_response_chunk.completion);
                                io::stdout().flush().unwrap();
                                output += &good_response_chunk.completion;
                            }
                        },
                        &_ => todo!()
                    }
                }
            },
            otherwise => panic!("received unexpected event type: {:?}", otherwise),
        }
    }
    Ok(())
}
