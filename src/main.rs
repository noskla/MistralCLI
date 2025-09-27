mod config;
mod constants;
mod types;
mod utils;

use crate::constants::DEFAULT_MODEL;
use crate::utils::make_mistral_request;
use std::{env, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    let mut model = DEFAULT_MODEL.to_string();
    let mut config_path: Option<&str> = None;
    let prompt: String;

    if args.len() < 2 {
        eprintln!("Usage: mistral [-c config_path] [-m model_name] <prompt>");
        return Err("Invalid arguments".into());
    }

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-c" => {
                if i + 1 < args.len() {
                    config_path = Some(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Missing value for -c option");
                    return Err("Invalid arguments".into());
                }
            }
            "-m" => {
                if i + 1 < args.len() {
                    model = format!("mistral-{}", args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Missing value for -m option");
                    return Err("Invalid arguments".into());
                }
            }
            _ => break,
        }
    }

    if let Ok(Some(config_model)) = crate::config::get_preferred_model(config_path) {
        model = config_model;
    }

    prompt = args[i..].join(" ");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    make_mistral_request(&client, &model, &prompt, config_path).await?;

    Ok(())
}
