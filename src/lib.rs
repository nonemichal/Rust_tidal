use std::error::Error;
use std::fs::File;
use std::collections::HashMap;
use clap::{Parser, Args, Subcommand, ValueEnum};
use base64::{Engine as _, engine::general_purpose};
use urlencoding;
use reqwest::Url;
use reqwest::header::{HeaderValue, HeaderMap};
use serde_json::Value;

#[macro_use]
extern crate lazy_static;

/// Program working with Tidal API
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Login to Tidal API and save a config file
    Login { client_id: String, client_secret: String },

    /// Send a query to search for content
    Search(SearchArgs),
}

#[derive(Args)]
pub struct SearchArgs {
    /// Search query
    query: String,

    /// Target search type
    #[arg(short, long, value_enum, default_value_t = TargetType::All)]
    target_type: TargetType,

    /// Pagination offset
    #[arg(short, long, default_value = "0")]
    offset: String,

    /// Page size
    #[arg(short, long, default_value = "10")]
    limit: String,

    /// ISO 3166-1 alpha-2 country code
    #[arg(short, long, default_value = "US")]
    country_code: String,

    /// Specify which popularity type to apply for query result
    #[arg(short, long, value_enum, default_value_t = Popularity::Worldwide)]
    popularity: Popularity,

    /// Flag to save content in a json file
    #[arg(short, long)]
    save_file: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Hash)]
pub enum TargetType {
    All,
    Artists,
    Albums,
    Tracks,
    Videos,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Hash)]
pub enum Popularity {
    Worldwide,
    Country,
}

lazy_static! {
    static ref TARGET_HASHMAP: HashMap<TargetType, &'static str> = {
        let mut m = HashMap::new();
        m.insert(TargetType::All, "");
        m.insert(TargetType::Artists, "ARTISTS");
        m.insert(TargetType::Albums, "ALBUMS");
        m.insert(TargetType::Tracks, "TRACKS");
        m.insert(TargetType::Videos, "VIDEOS");
        m
    };
}

lazy_static! {
    static ref POPULARITY_HASHMAP: HashMap<Popularity, &'static str> = {
        let mut m = HashMap::new();
        m.insert(Popularity::Worldwide, "WORLDWIDE");
        m.insert(Popularity::Country, "COUNTRY");
        m
    };
}


impl Cli {
    pub fn get_command(&self) -> &Commands {
        &self.command
    }

    /// Returns client ID and client Secret in HashMap
    /// If command is "login" gets arguments from user input
    /// If command is "search" gets arguments from json file and returns error when file does not exist
    pub fn get_login_args(&self) -> Result<HashMap<&str, String>, Box<dyn Error>> {
        let command = &self.command;
        let (id, secret): (String, String);

        match command {
            Commands::Login { client_id, client_secret } => {
                id = client_id.clone();
                secret = client_secret.clone();
            }
            Commands::Search(_args) => {
                if std::path::Path::new("config.json").exists() {
                    let file = std::fs::File::open("config.json")?;
                    let reader = std::io::BufReader::new(file);
                    let json: Value = serde_json::from_reader(reader)?;
                    let object = json.as_object().unwrap();
                    id = object["client_id"].as_str().unwrap().to_string();
                    secret = object["client_secret"].as_str().unwrap().to_string()
                } else {
                    return Err("Use login command to provide client ID and client secret".into());
                }
            }
        }

        Ok(HashMap::from([("client_id", id), ("client_secret", secret)]))
    }
}

impl SearchArgs {
    pub fn get_save_flag(&self) -> bool {
        self.save_file
    }

    pub fn get_target_type(&self) -> &TargetType {
        &self.target_type
    }

    /// Creates and returns HashMap of search arguments
    pub fn get_search_args(&self) -> HashMap<&str, String> {
        let mut search_args = HashMap::from([
            ("query", urlencoding::encode(&self.query.clone()).into_owned()),
            ("offset", self.offset.clone()),
            ("limit", self.limit.clone()),
            ("countryCode", self.country_code.clone()),
            ("popularity", POPULARITY_HASHMAP.get(&self.popularity).unwrap().to_string()),
        ]);

        if self.get_target_type() != &TargetType::All {
            search_args.insert("type", TARGET_HASHMAP.get(&self.get_target_type()).unwrap().to_string());
        }

        search_args
    }
}

/// Saves a json file with provided content and name
pub fn save_json(json: &Value, name: &str) -> Result<(), Box<dyn Error>> {
    serde_json::to_writer(&File::create(format!("{}.json", name))?, &json)?;

    Ok(())
}

/// Encodes client ID and client secret  as base64 string
fn encode_base64(client_id: &str, client_secret: &str) -> String {
    let string_to_encode = format!("{}:{}", client_id, client_secret);
    general_purpose::STANDARD.encode(string_to_encode)
}

/// Send posts request to get access token from Tidal API
/// If client ID or client secret is wrong returns error
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
    let data_obj = data.as_object().unwrap();

    if data_obj.contains_key("error") {
        return Err(data_obj["error_description"].as_str().unwrap().into());
    }

    let access_token = data["access_token"].as_str().unwrap().to_string();

    Ok(access_token)
}

/// Sends get request to get json data based on search arguments
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

/// Checks if sent request results in error
pub fn check_for_error(json: &Value) -> Result<(), Box<dyn Error>> {
    if json.as_object().unwrap().contains_key("errors") {
        let body = json.as_object().unwrap()["errors"].as_array().unwrap();
        let mut errors = String::new();

        for error in body {
            errors.push_str(error.as_object().unwrap()["detail"].as_str().unwrap());
        }

        return Err(errors.into());
    }

    Ok(())
}

/// Prints content of json file received by get request
/// Tidal API is in beta version, so some (or more) content may result in error 451
pub fn print_content(json: &Value, target_type: &TargetType) -> Result<(), Box<dyn Error>> {
    let mut targets = vec![];

    if target_type == &TargetType::All {
        targets.push(TARGET_HASHMAP.get(&TargetType::Artists).unwrap().to_string().to_lowercase());
        targets.push(TARGET_HASHMAP.get(&TargetType::Albums).unwrap().to_string().to_lowercase());
        targets.push(TARGET_HASHMAP.get(&TargetType::Tracks).unwrap().to_string().to_lowercase());
        targets.push(TARGET_HASHMAP.get(&TargetType::Videos).unwrap().to_string().to_lowercase());
    } else {
        targets.push(TARGET_HASHMAP.get(&target_type).unwrap().to_string().to_lowercase());
    }

    for target in targets {
        println!("\n{}: \n", target.to_uppercase());

        for data in json.as_object().unwrap()[&target].as_array().unwrap() {
            let status = data["status"].as_i64().unwrap();

            if status == 200 {
                let resource = data.as_object().unwrap()["resource"].as_object().unwrap();
                let content: String;

                if target == "artists" {
                    let name = resource["name"].as_str().unwrap();
                    content = format!("\t{}", name);
                } else {
                    let artists = resource["artists"].as_array().unwrap();
                    let title = resource["title"].as_str().unwrap();
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
                    if target != "tracks" {
                        let release_date = resource["releaseDate"].as_str().unwrap();
                        content = format!("{} - {} {}", artists_name, title, release_date);
                    } else {
                        content = format!("{} - {}", artists_name, title);
                    }
                }

                println!("\t{}\n", content);
            } else {
                println!("\tError {}: {}\n", status, data["message"].as_str().unwrap());
            }
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
