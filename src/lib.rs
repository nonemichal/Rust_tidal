
use std::error::Error;
use std::fs::File;
use base64::{Engine as _, engine::general_purpose};
use urlencoding;
use reqwest::Url;
use reqwest::header::{HeaderValue, HeaderMap};
use serde_json::Value;

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

pub async fn get_json_data(client: &reqwest::Client, access_token: &str, input: &str) -> Result<Value, Box<dyn Error>> {
    let bearer_token = format!("Bearer {}", &access_token);

    let type_param = std::borrow::Cow::Borrowed("ALBUMS");
    let offset_param = std::borrow::Cow::Borrowed("0");
    let limit_param = std::borrow::Cow::Borrowed("10");
    let country_code_param = std::borrow::Cow::Borrowed("PL");
    let popularity_param = std::borrow::Cow::Borrowed("WORLDWIDE");

    let encoded_input = urlencoding::encode(&input.trim());
    let url = Url::parse_with_params("https://openapi.tidal.com/search?",
                                                [("query", encoded_input),
                                                        ("type", type_param),
                                                        ("offset", offset_param),
                                                        ("limit", limit_param),
                                                        ("countryCode", country_code_param),
                                                        ("popularity", popularity_param)])?;

    let mut headers = HeaderMap::new();
    headers.insert("accept", HeaderValue::from_static("application/vnd.tidal.v1+json"));
    headers.insert("Authorization", HeaderValue::from_str(&bearer_token).unwrap());
    headers.insert("Content-Type", HeaderValue::from_static("application/vnd.tidal.v1+json"));

    let res = client
        .get(url)
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    let json: Value = serde_json::from_str(&res)?;

    Ok(json)
}

pub fn print_titles(json: &Value) -> Result<(), Box<dyn Error>> {
    for album in json.as_object().unwrap()["albums"].as_array().unwrap() {
        let status = album["status"].as_i64().unwrap();

        if status == 200 {
            let resource = album.as_object().unwrap()["resource"].as_object().unwrap();
            let artists = resource["artists"].as_array().unwrap();
            let artists_len = artists.len();
            let mut artists_name = String::new();
            let mut counter = 0;

            for artist in artists {
                artists_name.push_str(artist["name"].as_str().unwrap());
                if counter != 0 && counter != artists_len {
                    artists_name.push_str(", ");
                }

                counter += 1;
            }

            let album_name = resource["title"].as_str().unwrap();
            let release_date = resource["releaseDate"].as_str().unwrap();
            let content = format!("{} - {} {}", artists_name, album_name, release_date);

            println!("{}\n", content);
        } else {
            println!("Error {}: {}\n", status, album["message"].as_str().unwrap());
        }
    }

    Ok(())
}

pub fn save_json(json: &Value) -> Result<(), Box<dyn Error>> {
    serde_json::to_writer(&File::create("data.json")?, &json)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_test() {
        let client_id = String::from("9Qxzt81m6jPc8yiU");
        let client_secret = String::from("ACK3uEB9RkDj5NLWzbjND9JesAiGnUaW9SsGw94KxJ8=");

        assert_eq!(encode_base64(&client_id, &client_secret), "OVF4enQ4MW02alBjOHlpVTpBQ0szdUVCOVJrRGo1TkxXemJqTkQ5SmVzQWlHblVhVzlTc0d3OTRLeEo4PQ==");
    }
}