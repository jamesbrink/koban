use std::env;

use url::Url;

use crate::{KobanError, Result};

pub const DEFAULT_BASE_URL: &str = "https://invoicing.co";
pub const API_TOKEN_ENV: &str = "INVOICE_NINJA_API_TOKEN";
pub const BASE_URL_ENV: &str = "INVOICE_NINJA_BASE_URL";

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: Url,
    pub api_token: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let api_token = env::var(API_TOKEN_ENV).map_err(|_| KobanError::MissingToken)?;
        let base_url = env::var(BASE_URL_ENV).unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        Self::from_values(base_url, api_token)
    }

    pub fn from_values(base_url: impl AsRef<str>, api_token: impl Into<String>) -> Result<Self> {
        let api_token = api_token.into();
        if api_token.trim().is_empty() {
            return Err(KobanError::MissingToken);
        }

        let mut base_url =
            Url::parse(base_url.as_ref()).map_err(|source| KobanError::InvalidBaseUrl {
                value: base_url.as_ref().to_string(),
                source,
            })?;

        let is_local = matches!(base_url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
        if base_url.scheme() != "https" && !is_local {
            return Err(KobanError::InsecureBaseUrl {
                value: base_url.to_string(),
            });
        }

        if !base_url.path().ends_with('/') {
            let mut path = base_url.path().to_string();
            path.push('/');
            base_url.set_path(&path);
        }

        Ok(Self {
            base_url,
            api_token,
        })
    }
}
