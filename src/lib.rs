use std::error::Error;
use std::fs::File;
use std::collections::HashMap;
use clap::Parser;
use base64::{Engine as _, engine::general_purpose};
use urlencoding;
use reqwest::Url;
use reqwest::header::{HeaderValue, HeaderMap};
use serde_json::Value;

/// Program working with Tidal API
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Flag specifying whether login data should be saved to json file
    #[clap(short, long)]
    login_save: bool,

    /// Flag specifying whether searched data should be saved to json file
    #[clap(short, long)]
    data_save: bool,

    /// Client ID
    #[clap(long)]
    id: Option<String>,

    /// Client secret
    #[clap(long)]
    secret: Option<String>,

    /// Search query
    #[clap(short, long)]
    search: String,

    /// Target search type
    #[clap(long, default_value = "")]
    target_type: String,

    /// Pagination offset
    #[clap(long, default_value = "0")]
    offset: String,

    /// Page size
    #[clap(long, default_value = "10")]
    limit: String,

    /// ISO 3166-1 alpha-2 country code
    #[clap(long, default_value = "PL")]
    country_code: String,

    /// Specify which popularity type to apply for query result
    #[clap(long, default_value = "WORLDWIDE")]
    popularity: String,
}

impl Args {
    pub fn get_login_save(&self) -> bool {
        self.login_save
    }

    pub fn get_data_save(&self) -> bool {
        self.data_save
    }

    pub fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    pub fn get_secret(&self) -> Option<String> {
        self.secret.clone()
    }

    pub fn get_search_args(&self) -> HashMap<&str, String> {
        let mut args = HashMap::from([
            ("query", urlencoding::encode(self.search.clone().trim()).into_owned()),
            ("offset", self.offset.clone()),
            ("limit", self.limit.clone()),
            ("countryCode", self.country_code.clone()),
            ("popularity", self.popularity.clone()),
        ]);

        if !self.target_type.is_empty() {
            args.insert("type", self.target_type.clone());
        }

        args
    }
}

pub fn save_json(json: &Value, name: &str) -> Result<(), Box<dyn Error>> {
    serde_json::to_writer(&File::create(format!("{}.json", name))?, &json)?;

    Ok(())
}

pub fn get_id_and_secret(args: &Args) -> Result<(String, String), Box<dyn Error>> {
    let (client_id, client_secret): (String, String);

    if args.id.is_none() || args.secret.is_none() {
        if std::path::Path::new("login.json").exists() {
            let file = std::fs::File::open("login.json")?;
            let reader = std::io::BufReader::new(file);
            let json: Value = serde_json::from_reader(reader)?;
            let object = json.as_object().unwrap();

            client_id = object["client_id"].as_str().unwrap().to_string();
            client_secret = object["client_secret"].as_str().unwrap().to_string();
        } else {
            return Err("Client ID and secret must be given to connect to Tidal API".into());
        }
    } else {
        client_id = args.id.clone().unwrap().to_string();
        client_secret = args.secret.clone().unwrap().to_string();
    }

    Ok((client_id, client_secret))
}

fn encode_base64(client_id: &str, client_secret: &str) -> String {
    let string_to_encode = format!("{}:{}", client_id, client_secret);
    general_purpose::STANDARD.encode(string_to_encode)
}

pub async fn get_access_token(client: &reqwest::Client, client_id: &str, client_secret: &str) -> Result<String, Box<dyn Error>> {
    let encoded_string = encode_base64(client_id, client_secret);

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
    let access_token = data["access_token"].as_str().unwrap().to_string();

    Ok(access_token)
}

pub async fn get_json_data(client: &reqwest::Client, access_token: &str, input: &HashMap<&str, String>) -> Result<Value, Box<dyn Error>> {
    let bearer_token = format!("Bearer {}", &access_token);

    let url = Url::parse_with_params("https://openapi.tidal.com/search?", input.iter())?;

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
                if counter != 0 && counter != artists_len {
                    artists_name.push_str(", ");
                }

                artists_name.push_str(artist["name"].as_str().unwrap());

                counter += 1;
            }

            let album_name = resource["title"].as_str().unwrap();
            let release_date = resource["releaseDate"].as_str().unwrap();
            let content = format!("{} - {}\t{}", artists_name, album_name, release_date);

            println!("{}\n", content);
        } else {
            println!("Error {}: {}\n", status, album["message"].as_str().unwrap());
        }
    }

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