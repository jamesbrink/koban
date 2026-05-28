use std::{fs, path::PathBuf};

use reqwest::StatusCode;
use serde_json::Value;
use url::Url;

use crate::{Config, KobanError, Result, redact};

const REQUESTED_WITH: &str = "XMLHttpRequest";

pub struct ApiClient {
    config: Config,
    http: reqwest::Client,
}

impl ApiClient {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    pub fn endpoint(&self, path: &str, query: &[(String, String)]) -> Result<Url> {
        let mut url = self
            .config
            .base_url
            .join(path.trim_start_matches('/'))
            .map_err(|source| KobanError::InvalidEndpoint {
                path: path.to_string(),
                source,
            })?;

        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(
                query
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_str())),
            );
        }

        Ok(url)
    }

    pub async fn get_json(&self, path: &str, query: &[(String, String)]) -> Result<Value> {
        let url = self.endpoint(path, query)?;
        let endpoint = endpoint_label(&url);
        let response = self
            .http
            .get(url)
            .header("X-API-TOKEN", &self.config.api_token)
            .header("X-Requested-With", REQUESTED_WITH)
            .send()
            .await
            .map_err(|source| KobanError::Transport {
                message: redact(source.to_string(), &self.config.api_token),
            })?;

        let status = response.status();
        let body = response.text().await.map_err(|source| KobanError::Decode {
            message: redact(source.to_string(), &self.config.api_token),
        })?;

        if !status.is_success() {
            return Err(api_error(status, endpoint, body, &self.config.api_token));
        }

        serde_json::from_str(&body).map_err(|source| KobanError::Decode {
            message: redact(source.to_string(), &self.config.api_token),
        })
    }

    pub async fn post_json(
        &self,
        path: &str,
        query: &[(String, String)],
        body: &Value,
    ) -> Result<Value> {
        let url = self.endpoint(path, query)?;
        let endpoint = endpoint_label(&url);
        let response = self
            .http
            .post(url)
            .header("X-API-TOKEN", &self.config.api_token)
            .header("X-Requested-With", REQUESTED_WITH)
            .json(body)
            .send()
            .await
            .map_err(|source| KobanError::Transport {
                message: redact(source.to_string(), &self.config.api_token),
            })?;

        self.json_response(response, endpoint).await
    }

    pub async fn put_json(
        &self,
        path: &str,
        query: &[(String, String)],
        body: &Value,
    ) -> Result<Value> {
        let url = self.endpoint(path, query)?;
        let endpoint = endpoint_label(&url);
        let response = self
            .http
            .put(url)
            .header("X-API-TOKEN", &self.config.api_token)
            .header("X-Requested-With", REQUESTED_WITH)
            .json(body)
            .send()
            .await
            .map_err(|source| KobanError::Transport {
                message: redact(source.to_string(), &self.config.api_token),
            })?;

        self.json_response(response, endpoint).await
    }

    pub async fn delete_json(&self, path: &str, query: &[(String, String)]) -> Result<Value> {
        let url = self.endpoint(path, query)?;
        let endpoint = endpoint_label(&url);
        let response = self
            .http
            .delete(url)
            .header("X-API-TOKEN", &self.config.api_token)
            .header("X-Requested-With", REQUESTED_WITH)
            .send()
            .await
            .map_err(|source| KobanError::Transport {
                message: redact(source.to_string(), &self.config.api_token),
            })?;

        self.json_response(response, endpoint).await
    }

    pub async fn put_multipart(
        &self,
        path: &str,
        query: &[(String, String)],
        files: &[PathBuf],
    ) -> Result<Value> {
        let url = self.endpoint(path, query)?;
        let endpoint = endpoint_label(&url);
        let mut form = reqwest::multipart::Form::new();

        for path in files {
            let bytes = fs::read(path).map_err(|source| KobanError::File {
                message: format!("could not read {}: {source}", path.display()),
            })?;
            let file_name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "document".to_string());
            let part = reqwest::multipart::Part::bytes(bytes).file_name(file_name);
            form = form.part("documents[]", part);
        }

        let response = self
            .http
            .put(url)
            .header("X-API-TOKEN", &self.config.api_token)
            .header("X-Requested-With", REQUESTED_WITH)
            .multipart(form)
            .send()
            .await
            .map_err(|source| KobanError::Transport {
                message: redact(source.to_string(), &self.config.api_token),
            })?;

        self.json_response(response, endpoint).await
    }

    pub async fn get_bytes(&self, path: &str, query: &[(String, String)]) -> Result<Vec<u8>> {
        let url = self.endpoint(path, query)?;
        let endpoint = endpoint_label(&url);
        let response = self
            .http
            .get(url)
            .header("X-API-TOKEN", &self.config.api_token)
            .header("X-Requested-With", REQUESTED_WITH)
            .send()
            .await
            .map_err(|source| KobanError::Transport {
                message: redact(source.to_string(), &self.config.api_token),
            })?;

        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|source| KobanError::Decode {
                message: redact(source.to_string(), &self.config.api_token),
            })?;

        if status.is_success() {
            Ok(bytes.to_vec())
        } else {
            let body = String::from_utf8_lossy(&bytes).to_string();
            Err(api_error(status, endpoint, body, &self.config.api_token))
        }
    }

    async fn json_response(&self, response: reqwest::Response, endpoint: String) -> Result<Value> {
        let status = response.status();
        let body = response.text().await.map_err(|source| KobanError::Decode {
            message: redact(source.to_string(), &self.config.api_token),
        })?;

        if !status.is_success() {
            return Err(api_error(status, endpoint, body, &self.config.api_token));
        }

        serde_json::from_str(&body).map_err(|source| KobanError::Decode {
            message: redact(source.to_string(), &self.config.api_token),
        })
    }
}
pub(crate) fn api_error(
    status: StatusCode,
    endpoint: String,
    body: String,
    token: &str,
) -> KobanError {
    let message = serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|value| {
            first_json_string(&value, &["message"])
                .or_else(|| first_json_string(&value, &["error"]))
                .or_else(|| first_json_string(&value, &["errors"]))
        })
        .unwrap_or(body);

    KobanError::Api {
        status: status.as_u16(),
        endpoint,
        message: redact(message, token),
    }
}

fn first_json_string(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    match current {
        Value::String(value) => Some(value.clone()),
        Value::Array(items) => Some(
            items
                .iter()
                .map(|item| match item {
                    Value::String(value) => value.clone(),
                    other => other.to_string(),
                })
                .collect::<Vec<_>>()
                .join(", "),
        ),
        Value::Object(_) => Some(current.to_string()),
        _ => None,
    }
}

fn endpoint_label(url: &Url) -> String {
    match url.query() {
        Some(query) => format!("{}?{query}", url.path()),
        None => url.path().to_string(),
    }
}
