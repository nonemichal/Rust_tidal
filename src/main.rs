use std::error::Error;
use serde_json::{Value, json};
use clap::Parser;
use tidal::{get_id_and_secret, get_access_token, get_json_data, print_titles, save_json, Args};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.get_login_save() {
        if args.get_id().is_some() && args.get_secret().is_some() {
            let json: Value = json!({
                "client_id": args.get_id().clone().unwrap(),
                "client_secret": args.get_secret().clone().unwrap()
            });
            save_json(&json, "login")?;
        } else {
            return Err("Client ID and secret must be given to save login data".into());
        }
    }

    let (client_id, client_secret) = get_id_and_secret(&args)?;

    let input = args.get_search_args();
    let client = reqwest::Client::new();

    let access_token = get_access_token(&client, &client_id, &client_secret).await?;

    let json: Value = get_json_data(&client, &access_token, &input).await?;

    print_titles(&json)?;

    if args.get_data_save() {
        save_json(&json, "data")?;
    }


    Ok(())
}
