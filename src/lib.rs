
use base64::{Engine as _, engine::general_purpose};
use reqwest::Url;
use reqwest::header::{HeaderValue, HeaderMap};
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use urlencoding;

fn encode_base64(client_id: &String, client_secret: &String) -> String {
    let string_to_encode = format!("{}:{}", client_id, client_secret);
    general_purpose::STANDARD.encode(string_to_encode)
}

pub async fn get_access_token(client: &reqwest::Client, client_id: &String, client_secret: &String) -> Result<String, Box<dyn Error>> {
    let encoded_string = encode_base64(&client_id, &client_secret);

    let params = [("grant_type", "client_credentials")];

    let result = client
        .post("https://auth.tidal.com/v1/oauth2/token")
        .header("Authorization", &format!("Basic {}", encoded_string))
        .form(&params)
        .send()
        .await?
        .text()
        .await?;

    let data: Value = serde_json::from_str(&result)?;
    let access_token = String::from(data["access_token"].as_str().unwrap());

    Ok(access_token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_test () {
        let client_id = String::from("9Qxzt81m6jPc8yiU");
        let client_secret = String::from("ACK3uEB9RkDj5NLWzbjND9JesAiGnUaW9SsGw94KxJ8=");

        assert_eq!(encode_base64(&client_id, &client_secret), "OVF4enQ4MW02alBjOHlpVTpBQ0szdUVCOVJrRGo1TkxXemJqTkQ5SmVzQWlHblVhVzlTc0d3OTRLeEo4PQ==");
    }
}