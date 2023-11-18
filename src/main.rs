use std::error::Error;
use serde_json::{Value, json};
use clap::Parser;
use tidal::{get_access_token, get_json_data, print_content, save_json, check_for_error, Cli, Commands};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.get_command() {
        Commands::Login { client_id, client_secret } => {
            let json: Value = json!({
                "client_id": client_id,
                "client_secret": client_secret
            });

            save_json(&json, "config")?;
            println!("Client ID and secret have been saved in the config.json file");

            return Ok(());
        }
        Commands::Search(args) => {
            let login_args = cli.get_login_args()?;
            let client_id = login_args["client_id"].clone();
            let client_secret = login_args["client_secret"].clone();

            let input = args.get_search_args()?;
            let client = reqwest::Client::new();
            let access_token = get_access_token(&client, &client_id, &client_secret).await?;

            let json: Value = get_json_data(&client, &access_token, &input).await?;

            check_for_error(&json)?;

            print_content(&json, &args.get_target_type())?;

            if args.get_save_flag() {
                save_json(&json, "data")?;
            }
        }
    }

    Ok(())
}
