use clap::Parser;
use reqwest::blocking::{Body, Client};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::option::Option;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    /// IP address or hostname of the Vault server
    #[arg(long, env, default_value_t = String::from("localhost"))]
    vault_api_host: String,

    /// Port that the Vault server API listens to
    #[arg(short, long, env, default_value_t = 8200)]
    vault_api_port: u16,

    /// Create a new, empty Vault
    #[arg(short, long, env, default_value_t = false)]
    bootstrap: bool,

    /// URL for downloading the snapshot for restoration
    #[arg(short, long, env)]
    snapshot_url: Option<Url>,

    /// Key for unsealing the Vault
    #[arg(short, long, env)]
    unseal_key: Option<String>,
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

#[derive(Serialize, Deserialize, Debug)]
struct UnsealRequest {
    key: String,
    reset: Option<bool>,
    migrate: Option<bool>,
}

fn main() {
    let args = Args::parse();
    let client = Client::builder().build().expect("failed to build client");
    let base_url = format!("http://{}:{}/v1", args.vault_api_host, args.vault_api_port);

    let init_response = initialize(&client, &base_url).expect("failed to initialize");
    if !args.bootstrap {
        let snapshot_url = &args.snapshot_url.expect("URL prompt not implemented yet");
        let snapshot_body =
            download_snapshot(args.bootstrap, snapshot_url).expect("error downloading snapshot");
        restore(&client, &base_url, snapshot_body.unwrap()).expect("failed to restore");
    }

    let unseal_key = if args.bootstrap {
        init_response.keys.get(0).expect("expected exactly one key")
    } else {
        args.unseal_key
            .as_ref()
            .expect("unseal key prompt not implemented yet")
    };

    unseal(&client, &base_url, unseal_key).expect("failed to unseal");
}

fn download_snapshot(bootstrap: bool, snapshot_url: &Url) -> Result<Option<Body>, std::io::Error> {
    if bootstrap {
        Ok(None)
    } else {
        match snapshot_url.scheme() {
            "file" => {
                let snapshot_pathbuf = snapshot_url
                    .to_file_path()
                    .expect("Could not get path from file URL");
                let snapshot_path = snapshot_pathbuf.as_path();
                let snapshot_file = File::open(snapshot_path)?;
                Ok(Some(Body::from(snapshot_file)))
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{} scheme not supported", snapshot_url.scheme()),
            )),
        }
    }
}

fn initialize(client: &Client, base_url: &String) -> Result<Init, reqwest::Error> {
    let seal_status = client
        .get(format!("{base_url}/sys/seal-status"))
        .send()
        .expect("failed to get seal status")
        .json::<SealStatus>()?;

    if seal_status.initialized {
        panic!("Already initialized!!");
    }
    let init_resp = client
        .post(format!("{base_url}/sys/init"))
        .json(&InitRequest {
            secret_shares: 1,
            secret_threshold: 1,
        })
        .send()
        .expect("failed to post init request")
        .json::<Init>()?;

    let unseal_resp = client
        .post(format!("{base_url}/sys/unseal"))
        .json(&UnsealRequest {
            key: init_resp.keys[0].clone(),
            migrate: None,
            reset: None,
        })
        .send()
        .expect("failed to post unseal request")
        .json::<SealStatus>();

    Ok(init_resp)
}

fn restore(client: &Client, base_url: &String, snapshot_body: Body) -> Result<(), reqwest::Error> {
    let snapshot_response = client
        .post(format!("{base_url}/sys/storage/raft/snapshot-force"))
        .body(snapshot_body)
        .send()
        .expect("failed to post snapshot request");
    Ok(())
}

fn unseal(client: &Client, base_url: &String, unseal_key: &String) -> Result<(), reqwest::Error> {
    let seal_status = client
        .get(format!("{base_url}/sys/seal-status"))
        .send()
        .expect("failed to get seal status")
        .json::<SealStatus>()?;

    if seal_status.sealed {
        let unseal_resp = client
            .post(format!("{base_url}/sys/unseal"))
            .json(&UnsealRequest {
                key: unseal_key.to_string(),
                migrate: None,
                reset: None,
            })
            .send()
            .expect("failed to post unseal request")
            .json::<SealStatus>();
    }
    Ok(())
}

fn snapshot(client: &Client, base_url: &String, token: &String) -> Result<(), reqwest::Error> {
    Ok(())
}
