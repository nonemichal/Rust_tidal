use serde_json::Value;
use tidal::{get_access_token, get_json_data, print_titles, save_json};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    println!("Search: ");
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let client_id = String::from("gcjYbogNbf4qm7LQ");
    let client_secret = String::from("lHANoOjCStwsONMIeGyiT0aef0BmeEemfmIQq7BNqH8=");
    let client = reqwest::Client::new();

    let access_token = get_access_token(&client, &client_id, &client_secret).await?;

    let json: Value = get_json_data(&client, &access_token, &input).await?;

    print_titles(&json)?;

    save_json(&json)?;


    Ok(())
}
