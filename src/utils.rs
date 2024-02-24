use figlet_rs::FIGfont;
use clap::Parser;

// ######################################## ARGUMENT PARSING
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(value_enum)]
    #[arg(short,long)]
    pub model_id: ArgModels,
}

#[derive(clap::ValueEnum, Clone)]
pub enum ArgModels {
    Llama270b,
    CohereCommand,
    ClaudeV2,
    ClaudeV21,
    Jurrasic2Ultra,
}

impl ArgModels {
    pub fn to_str(&self) -> &'static str {
        match self {
            ArgModels::ClaudeV2 => "anthropic.claude-v2",
            ArgModels::ClaudeV21 => "anthropic.claude-v2:1",
            ArgModels::Llama270b => "meta.llama2-70b-chat-v1",
            ArgModels::CohereCommand => "cohere.command-text-v14",
            ArgModels::Jurrasic2Ultra => "ai21.j2-ultra-v1",
        }
    }
}
// ######################################## END ARGUMENT PARSING

pub fn hello_header(s: &str) {

    let ansi_font = FIGfont::from_file("resources/ansishadow.flf").unwrap();
    let hello = ansi_font.convert(s);
    println!("{}", hello.unwrap());

}
