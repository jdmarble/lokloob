use std::thread;
use std::time::Duration;
use clap::{Parser, Subcommand};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct LokLoob {
    #[command(subcommand)]
    command: Commands,

    /// IP address or hostname of the Vault server
    #[arg(long, env, default_value_t = String::from("localhost"))]
    address: String,

    /// Port that the Vault server API listens to
    #[arg(short, long, env, default_value_t = 8200)]
    port: u16,
}

#[derive(Subcommand)]
enum Commands {
    /// Wait for the Vault server to start and respond on the API port
    WaitForServer {},
    /// Restore a Vault server from a backup
    Restore {},
}

#[derive(Serialize, Deserialize, Debug)]
struct SealStatus {
    r#type: String,
    initialized: bool,
    sealed: bool,
    t: u8,
    n: u8,
    progress: u8,
    nonce: String,
    version: String,
    build_date: String,
    migration: bool,
    recovery_seal: bool,
    storage_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct InitRequest {
    secret_shares: u8,
    secret_threshold: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct Init {
    keys: Vec<String>,
    keys_base64: Vec<String>,
    root_token: String,
}

fn main() -> Result<(), reqwest::Error> {
    let args = LokLoob::parse();

    let client = Client::builder().build()?;
    let base_url = format!("http://{}:{}/v1", args.address, args.port);

    match &args.command {
        Commands::WaitForServer {} => {
            wait_for_server(&client, &base_url)
        },
        Commands::Restore {} => {
            wait_for_server(&client, &base_url)?;
            restore(&client, &base_url)
        },
    }
}

fn wait_for_server(client: &Client, base_url: &String) -> Result<(), reqwest::Error> {
    loop {
        match attempt_connection(client, base_url) {
            Ok(_) => {
                break;
            }
            Err(_) => {
                print!(".");
                thread::sleep(Duration::from_secs(1));
            }
        }
    };
    Ok(())
}

fn attempt_connection(client: &Client, base_url: &String) -> Result<(), reqwest::Error> {
    client
        .head(format!("{base_url}/v1/sys/health"))
        .send()
        .map(|_result_code| ()) // Any result is fine for checking that the server is running.
}

fn restore(client: &Client, base_url: &String) -> Result<(), reqwest::Error> {

    let _status1 = client
        .get(format!("{base_url}/sys/seal-status"))
        .send()
        .expect("failed to get seal status")
        .json::<SealStatus>();

    let _init_resp = client
        .post(format!("{base_url}/sys/init"))
        .json(&InitRequest {
            secret_shares: 1,
            secret_threshold: 1,
       })
        .send()
        .expect("failed to post init request")
        .json::<Init>();

    let _status2 = client
        .get(format!("{base_url}/sys/seal-status"))
        .send()
        .expect("failed to get seal status")
        .json::<SealStatus>();

    Ok(())
}
