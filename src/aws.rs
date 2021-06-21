use std::{
    collections::HashMap,
    io::Write,
    fs::OpenOptions,
    process::{
        Command,
    },
    env,
};


use color_eyre::{Result, eyre::{Context, eyre}};
use serde_derive::{
    Serialize,
    Deserialize,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
struct Credentials {
    access_key_id: String,
    secret_access_key: String,
    session_token: String,
    expiration: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
struct AssumedRoleUser {
    assumed_role_id: String,
    arn: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
struct StsResponse {
    credentials: Credentials,
    assumed_role_user: AssumedRoleUser
}

#[derive(Serialize, Deserialize, Debug)]
struct CredentialsConfigEntry {
    aws_access_key_id: String,
    aws_secret_access_key: String,
    aws_session_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AwsConfig {
    sso_start_url: String,
    sso_region: String,
    sso_account_id: String,
    sso_role_name: String,
    region: String,
}

#[derive(Debug)]
pub struct AwsClient {
    root: String,
    credentials: HashMap<String, CredentialsConfigEntry>,
    aws_config: HashMap<String, AwsConfig>,
}

impl Default for CredentialsConfigEntry {
    fn default() -> Self {
        CredentialsConfigEntry {
            aws_access_key_id: "Default".to_owned(),
            aws_secret_access_key: "Default".to_owned(),
            aws_session_token: "Default".to_owned(),
        }
    }
}

impl AwsClient {

    fn check_logged_in(&self, profile: &str) -> Result<&Self> {
        let logged_in = Command::new("aws")
            .args(&[
                "iam",
                "list-roles",
                "--profile",
                profile,
                "--no-cli-pager"
            ])
            .stdout(std::process::Stdio::null())
            .status()?;

        if logged_in.success() {
            return Ok(self);
        }

        Err(eyre!("You are not logged in"))
    }

    fn log_in(&mut self, profile: &str) -> Result<&mut Self> {
        let log_in = Command::new("aws")
            .args(&[
                "sso",
                "login",
                "--profile",
                profile
            ])
            .status()?;

        if log_in.success() {
            return Ok(self)
        }

        Err(eyre!("Could not log in"))
    }

    pub fn new() -> Result<Self> {
        let root = format!("{}/.aws", env::var("HOME")?);
        let aws_config = load_config(&root)?;
        let credentials = load_credentials(&root)?;

        let client = AwsClient {
            root,
            credentials,
            aws_config
        };

        Ok(client)
    }

    pub fn assume(&mut self, profile: &str, role_arn: &str, name: &str) -> Result<&mut Self> {
        if let Err(_) = self.check_logged_in(profile) {
            self.log_in(profile)?;
        }

        let response = self.get_credentials(profile, role_arn, name)?;

        self.write_credentials(profile, &response)?;

        Ok(self)
    }

    fn get_credentials(&self, profile: &str, role_arn: &str, name: &str) -> Result<StsResponse> {
        let output = Command::new("aws")
            .args(&[
                "sts",
                "assume-role",
                "--profile",
                profile,
                "--role-arn",
                role_arn,
                "--role-session-name",
                name
            ])
            .output()?;

        if output.stderr.len() > 0 {
            return Err(
                eyre!(
                    "Role could not be assumed: {}",
                    String::from_utf8(output.stderr)?));
        }

        serde_json::from_str(
            String::from_utf8(output.stdout)
            .wrap_err("Stdout not valid UTF-8")?
            .as_str()
        ).wrap_err("Could not handle output")
    }

    fn write_credentials(&mut self, profile: &str, response: &StsResponse) -> Result<()> {
        self.credentials.entry(profile.to_owned()).or_default();
        self.credentials.entry(profile.to_owned()).and_modify(|c| {
            c.aws_access_key_id = response.credentials.access_key_id.clone();
            c.aws_secret_access_key = response.credentials.secret_access_key.clone();
            c.aws_session_token = response.credentials.session_token.clone();
        });

        let cred_string = serde_ini::to_string(&self.credentials)?;
        let path = format!("{}/credentials", &self.root);
        let mut cred_file = OpenOptions::new().write(true).open(path)?;
        cred_file.write_all(cred_string.as_bytes())?;
        cred_file.flush()?;
        Ok(())
    }
}

fn load_config(path: &String) -> Result<HashMap<String, AwsConfig>> {
    let aws_config = {
        let config_file = std::fs::File::open(
        std::path::Path::new(
            format!(
                "{}/config",
                path,
            ).as_str()
        ))?;
        let reader = std::io::BufReader::new(config_file);

        serde_ini::de::from_bufread(reader)?
    };
    Ok(aws_config)
}

fn load_credentials(path: &String) -> Result<HashMap<String, CredentialsConfigEntry>> {
    let aws_credentials = {
        let config_file = std::fs::File::open(
        std::path::Path::new(
            format!(
                "{}/credentials",
                path,
            ).as_str()
        ))?;
        let reader = std::io::BufReader::new(config_file);

        serde_ini::de::from_bufread(reader).map_err(|e| {
            println!("Could not read credentials file:");
            println!("{}", e);
        }).unwrap()
    };
    Ok(aws_credentials)
}
