
use base64::{Engine as _, engine::general_purpose};
use std::error::Error;

pub fn encode_base64(client_id: &String, client_secret: &String) -> String {
    let string_to_encode = format!("{}:{}", client_id, client_secret);

    general_purpose::STANDARD.encode(string_to_encode)
}

pub async fn get_access_token(encoded_string: &String) -> Result<(), Box<dyn Error>> {
    let encoded_arg = format!("Basic {}", encoded_string);

    println!("{}", encoded_arg);

    let client = reqwest::Client::new()
        .post("https://auth.tidal.com/v1/oauth2/token")
        .header("Authorization", encoded_arg)
        .body("grant_type=client_credentials")
        .send()
        .await?;

    println!("{:#?}", client);

    Ok(())
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