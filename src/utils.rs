use crate::config::get_api_key;
use crate::constants::MISTRAL_API_URL;
use crate::types::{MessageRole, MistralApiResponse, MistralRequestBody, ModelListResponse};
use futures::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::io::{self, Write};
use std::{error::Error};

pub async fn make_mistral_request(
    client: &reqwest::Client,
    model: &str,
    prompt: &str,
    config_path: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let api_key = get_api_key(config_path)?;
    let headers = HeaderMap::from_iter(vec![(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))?,
    )]);
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.yellow} {msg}"),
    );
    spinner.enable_steady_tick(100);
    spinner.set_message("Generating response...");

    let request_body = MistralRequestBody {
        model: model.to_string(),
        messages: vec![MessageRole {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
        stream: true,
    };

    let response = client
        .post(MISTRAL_API_URL)
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;

    spinner.finish_and_clear();
    if response.status().is_success() {
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            let chunk_str = String::from_utf8(chunk.to_vec())?;

            for line in chunk_str.split('\n') {
                let line = line.trim_start_matches("data: ").trim();
                if !line.is_empty() {
                    match serde_json::from_str::<MistralApiResponse>(line) {
                        Ok(api_response) => {
                            for choice in api_response.choices {
                                if let Some(content) = choice.delta.content {
                                    print!("{}", content);
                                    io::stdout().flush().unwrap();
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    } else {
        spinner.finish_with_message("Error!");
        return Err("An unexpected error has occurred".into());
    }
    println!();

    Ok(())
}

pub async fn list_models(api_key: &str) -> Result<ModelListResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let url = "https://api.mistral.ai/v1/models";
    let headers = HeaderMap::from_iter(vec![(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))?,
    )]);

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let models: ModelListResponse = serde_json::from_str(&body)?;
        Ok(models)
    } else {
        Err("Failed to fetch models".into())
    }
}
