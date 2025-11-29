use cli_select::Select;
use ini::Ini;
use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Command;

#[derive(serde::Deserialize)]
struct MfaDevicesJson {
    #[serde(rename = "MFADevices")]
    mfa_devices: Vec<MfaDevice>,
}

#[derive(serde::Deserialize)]
struct MfaDevice {
    #[serde(rename = "SerialNumber")]
    serial_number: String,
}

#[derive(serde::Deserialize)]
struct CredentialsJson {
    #[serde(rename = "Credentials")]
    credentials: Credentials,
}

#[derive(serde::Deserialize, Debug)]
struct Credentials {
    #[serde(rename = "AccessKeyId")]
    access_key_id: String,
    #[serde(rename = "SecretAccessKey")]
    secret_access_key: String,
    #[serde(rename = "SessionToken")]
    session_token: String,
}

fn fetch_accounts(path: &PathBuf) -> Vec<String> {
    let value = Ini::load_from_file(&path).expect("Failed to load credentials file");

    let mut o = value
        .iter()
        .filter_map(|item| item.0)
        .filter(|item| !item.contains("_mfa-authorized"))
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    o.sort();
    o
}

fn fetch_mfa_devices(account_name: &str) -> Vec<String> {
    let mut mfa_cmd = Command::new("aws");
    let mfa_output = mfa_cmd
        .args(vec!["iam", "list-mfa-devices", "--profile", account_name])
        .output()
        .expect("Failed getting mfa devices");
    let mfa_devices: MfaDevicesJson =
        serde_json::from_slice(&mfa_output.stdout).expect("Failed parsing mfa device output");
    mfa_devices
        .mfa_devices
        .iter()
        .map(|item| item.serial_number.clone())
        .collect::<Vec<_>>()
}

fn fetch_credentials(account_name: &str, mfa_arn: &str, token: &str) -> Credentials {
    let mut session_cmd = Command::new("aws");
    let session_output = session_cmd
        .args(vec![
            "sts",
            "get-session-token",
            "--profile",
            account_name,
            "--serial-number",
            mfa_arn,
            "--token-code",
            token,
        ])
        .output()
        .expect("Failed");

    let credentials: CredentialsJson =
        serde_json::from_slice(&session_output.stdout).expect("Failed parsing credentials output");

    credentials.credentials
}

fn write_profile(
    path: &PathBuf,
    credentials: &Credentials,
    account_name: &str,
) -> std::io::Result<()> {
    let key = format!("{}_mfa-authorized", account_name);

    let mut conf = Ini::load_from_file(&path).expect("Failed to load credentials file");
    conf.with_section(Some(key.to_owned()))
        .set("aws_access_key_id", &credentials.access_key_id)
        .set("aws_secret_access_key", &credentials.secret_access_key)
        .set("aws_session_token", &credentials.session_token);

    // Write back to disk
    conf.write_to_file(&path)?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut path = PathBuf::from(env::var("HOME").unwrap());
    path.push(".aws/credentials");

    let accounts = fetch_accounts(&path);
    println!("Select a profile:");
    let mut select = Select::new(&accounts, std::io::stdout());
    let account_name = select.start();
    println!("");

    println!("Loading mfa devices, please wait...");
    let mfa_devices = fetch_mfa_devices(account_name);
    println!("Select a mfa device:");
    let mut select = Select::new(&mfa_devices, std::io::stdout());
    let mfa_arn = select.start();
    println!("");

    print!("Enter token: ");
    io::stdout().flush().unwrap();
    let stdin = io::stdin();
    let token = stdin.lock().lines().next().unwrap().unwrap();
    println!("");

    println!("Getting session...");
    let credentials = fetch_credentials(account_name, mfa_arn, &token);

    write_profile(&path, &credentials, &account_name)?;

    println!("Success");
    Ok(())
}
