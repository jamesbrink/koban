use httpmock::{
    Method::{GET, POST, PUT},
    MockServer,
};
use reqwest::StatusCode;

use crate::{
    api::{ApiClient, api_error},
    *,
};

mod api;
mod config;
mod models;
