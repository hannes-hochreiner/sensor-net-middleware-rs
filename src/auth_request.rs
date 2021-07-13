extern crate reqwest;
use chrono::prelude::*;
use chrono::Duration;
use serde_json::{json, Value};
use std::error::Error;
use std::fmt;

pub struct AuthRequestConfig {
    pub client_id: String,
    pub client_secret: String,
    pub audience: String,
    pub tenant: String,
    pub region: String,
    pub endpoint: String,
}

pub struct AuthRequest<'a> {
    config: &'a AuthRequestConfig,
    token: String,
    expiration: DateTime<Utc>,
}

impl AuthRequest<'_> {
    pub fn new(config: &AuthRequestConfig) -> AuthRequest {
        AuthRequest {
            config: config,
            token: String::from(""),
            expiration: Utc::now() - Duration::seconds(10),
        }
    }

    pub async fn send_message(&mut self, message: String) -> Result<(), Box<dyn Error>> {
        if self.expiration < Utc::now() {
            self.update_token().await?;
        }

        let client = reqwest::Client::new();
        match client
            .put(&self.config.endpoint)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(message)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(error) => Err(Box::new(error)),
        }
    }

    async fn update_token(&mut self) -> Result<(), Box<dyn Error>> {
        let body = json!({
            "client_id": self.config.client_id,
            "client_secret": self.config.client_secret,
            "audience": self.config.audience,
            "grant_type":"client_credentials"
        });
        let client = reqwest::Client::new();
        let res = client
            .post(&format!(
                "https://{}.{}.auth0.com/oauth/token",
                self.config.tenant, self.config.region
            ))
            .header("content-type", "application/json")
            .body(body.to_string())
            .send()
            .await?
            .text_with_charset("utf-8")
            .await?;
        let v: Value = serde_json::from_str(&res)?;

        self.token = match v["access_token"].as_str() {
            Some(val) => Ok(String::from(val)),
            None => Err(AuthRequestError {
                description: String::from("error extracting access token"),
            }),
        }?;
        self.expiration =
            Utc::now() + Duration::seconds(v["expires_in"].as_i64().unwrap_or(0) - 10);

        Ok(())
    }
}

#[derive(Debug)]
struct AuthRequestError {
    description: String,
}

impl Error for AuthRequestError {}

impl fmt::Display for AuthRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AuthRequestError: {}", self.description)
    }
}
