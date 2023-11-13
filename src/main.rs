use reqwest::Url;
use reqwest::header::{HeaderValue, HeaderMap};
use serde_json::Value;
use tidal::get_access_token;
use std::error::Error;
use std::fs::File;
use urlencoding;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    println!("Search: ");
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let type_param = std::borrow::Cow::Borrowed("ALBUMS");
    let offset_param = std::borrow::Cow::Borrowed("0");
    let limit_param = std::borrow::Cow::Borrowed("10");
    let country_code_param = std::borrow::Cow::Borrowed("PL");
    let popularity_param = std::borrow::Cow::Borrowed("WORLDWIDE");

    let encoded_input = urlencoding::encode(&input.trim());
    let url = Url::parse_with_params("https://openapi.tidal.com/search?",
                                                                &[("query", encoded_input),
                                                                        ("type", type_param),
                                                                        ("offset", offset_param),
                                                                        ("limit", limit_param),
                                                                        ("countryCode", country_code_param),
                                                                        ("popularity", popularity_param)])?;

    let client_id = String::from("gcjYbogNbf4qm7LQ");
    let client_secret = String::from("lHANoOjCStwsONMIeGyiT0aef0BmeEemfmIQq7BNqH8=");
    let client = reqwest::Client::new();

    let access_token = get_access_token(&client, &client_id, &client_secret).await?;
    let bearer_token = format!("Bearer {}", &access_token);


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

    let res_json: Value = serde_json::from_str(&res)?;

    //serde_json::to_writer(&File::create("data.json")?, &res_json)?;

    for album in res_json.as_object().unwrap()["albums"].as_array().unwrap() {
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