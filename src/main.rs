use reqwest::header::{self, HeaderName, HeaderValue, HeaderMap};
use serde_json::Value;
use tidal::{encode_base64, get_access_token};
use std::{error::Error, collections::HashMap};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client_id = String::from("9Qxzt81m6jPc8yiU");
    let client_secret = String::from("ACK3uEB9RkDj5NLWzbjND9JesAiGnUaW9SsGw94KxJ8=");
    let encoded_string = encode_base64(&client_id, &client_secret);

    let encoded_arg = format!("Basic {}", encoded_string);

    let params = [("grant_type", "client_credentials")];

    let client = reqwest::Client::new();

    let res = client
        .post("https://auth.tidal.com/v1/oauth2/token")
        .header("Authorization", &encoded_arg)
        .form(&params)
        .send()
        .await?
        .text()
        .await?;

    let data: Value = serde_json::from_str(&res)?;

    let access_token = data["access_token"].as_str().unwrap();
    let bearer_token = format!("Bearer {}", &access_token);

    println!("{}", bearer_token);


    let mut headers = HeaderMap::new();
    headers.insert("accept", HeaderValue::from_static("application/vnd.tidal.v1+json"));
    headers.insert("Authorization", HeaderValue::from_str(&bearer_token).unwrap());
    headers.insert("Content-Type", HeaderValue::from_static("application/vnd.tidal.v1+json"));

    let res = client
        .get("https://openapi.tidal.com/albums/59727856?countryCode=US")
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    let res_json: Value = serde_json::from_str(&res)?;

    println!("{:?}", res_json);


    Ok(())
}