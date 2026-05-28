use std::path::PathBuf;

use httpmock::{
    Method::{GET, POST, PUT},
    MockServer,
};
use reqwest::StatusCode;
use serde_json::Value;

use crate::{
    api::{ApiClient, api_error},
    cli::*,
    commands::*,
    invoice::*,
    render::*,
    *,
};

mod api;
mod commands;
mod config;
mod invoice;
mod render;
fn empty_payload_args() -> InvoicePayloadArgs {
    InvoicePayloadArgs {
        data: None,
        data_file: None,
        stdin: false,
        client_id: None,
        date: None,
        due_date: None,
        number: None,
        po_number: None,
        public_notes: None,
        private_notes: None,
        terms: None,
        footer: None,
        project_id: None,
        line_items: Vec::new(),
    }
}

fn empty_trigger_args() -> InvoiceTriggerArgs {
    InvoiceTriggerArgs {
        send_email: false,
        mark_sent: false,
        paid: false,
        amount_paid: None,
        cancel: false,
        save_default_footer: false,
        save_default_terms: false,
        retry_e_send: false,
    }
}
