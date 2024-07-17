use serde::{ser::SerializeStruct, Deserialize, Serialize};

#[derive(Debug)]
pub enum ClaudeV3ContentEnum {
    TextContent(ClaudeV3TextContent),
    ImageContent(ClaudeV3ImageContent),
}

// NOTE: This had to be implemented separately due to the way the message for ClaudeV3
// need to be made. The issue was that both the TextContent and ImageContent are different
// structs, but need to sit under the content json array, without its struct names.
// BEFORE: content: [ text_content: { // ... }, image_content: { // ...}]
// AFTER: content: [ { // ... }, { // ...}]
impl Serialize for ClaudeV3ContentEnum {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            ClaudeV3ContentEnum::TextContent(ref tc) => {
                let mut state = serializer.serialize_struct("Content", 2)?;
                state.serialize_field("type", &tc.content_type)?;
                if let Some(ref text) = tc.text {
                    state.serialize_field("text", text)?;
                }
                state.end()
            }
            ClaudeV3ContentEnum::ImageContent(ref ic) => {
                let mut state = serializer.serialize_struct("Content", 2)?;
                state.serialize_field("type", &ic.content_type)?;
                if let Some(ref source) = ic.source {
                    state.serialize_field("source", source)?;
                }
                state.end()
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClaudeV3Config {
    pub anthropic_version: String,
    pub max_tokens: i32,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub role: String,
    #[serde(rename = "type")]
    pub default_content_type: String,
}

#[derive(serde::Serialize, Debug)]
pub struct ClaudeV3Body {
    pub anthropic_version: String,
    pub max_tokens: i32,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub messages: Vec<ClaudeV3Message>,
}

#[derive(serde::Serialize, Debug)]
pub struct ClaudeV3Message {
    pub role: String,
    pub content: Option<Vec<ClaudeV3ContentEnum>>,
}

#[derive(serde::Serialize, Debug)]
pub struct ClaudeV3Content {
    //#[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<ClaudeV3TextContent>,
    //#[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_content: Option<ClaudeV3ImageContent>,
}

#[derive(serde::Serialize, Debug)]
pub struct ClaudeV3TextContent {
    // renaming content_type to type due to type being a keyword
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}

#[derive(serde::Serialize, Debug)]
pub struct ClaudeV3ImageContent {
    // renaming content_type to type due to type being a keyword
    #[serde(rename = "type")]
    pub content_type: String,
    pub source: Option<ClaudeImageSource>,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct ClaudeImageSource {
    #[serde(rename = "type")]
    pub image_type: String,
    pub media_type: String,
    pub data: String,
}

impl ClaudeV3Body {
    pub fn new(
        anthropic_version: String,
        max_tokens: i32,
        temperature: f32,
        top_p: f32,
        top_k: i32,
        role: String,
        _content_type: String,
        text: Option<String>,
        image_sources: Option<Vec<ClaudeImageSource>>,
    ) -> ClaudeV3Body {
        let mut content = Vec::new();
        if text.is_some() {
            let text_content = ClaudeV3TextContent {
                content_type: "text".to_string(),
                text,
            };
            content.push(ClaudeV3ContentEnum::TextContent(text_content));
        }

        if let Some(images) = image_sources {
            for image in images {
                let image_content = ClaudeV3ImageContent {
                    content_type: "image".to_string(),
                    source: Some(image),
                };
                content.push(ClaudeV3ContentEnum::ImageContent(image_content));
            }
        }
        let message = ClaudeV3Message {
            role,
            content: Some(content),
        };
        ClaudeV3Body {
            anthropic_version,
            temperature,
            top_p,
            top_k,
            max_tokens,
            messages: vec![message],
        }
    }
}

// NOTE: This is just dead code now, as we are not using structs to deserialize the data
// rather I am using serde_json::Value to return specific elements due to the complexity of
// the response coming from Claude V3
#[derive(serde::Deserialize, Debug)]
pub struct ClaudeV3Response {
    pub content: Vec<ClaudeV3ResponseContent>,
}

#[derive(serde::Deserialize, Debug)]
pub struct ClaudeV3ResponseContent {
    // renaming content_type to type due to type being a keyword
    //#[serde(rename="type")]
    //pub completion_type: String,
    pub text: String,
}
