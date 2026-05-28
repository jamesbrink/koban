use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use serde_json::{Value, json};

use crate::{
    KobanError, Result,
    cli::{InvoicePayloadArgs, InvoiceTriggerArgs, WriteSafetyArgs},
};

pub(crate) fn require_confirmation(operation: &str, safety: &WriteSafetyArgs) -> Result<()> {
    if safety.dry_run || safety.yes {
        Ok(())
    } else {
        Err(KobanError::ConfirmationRequired {
            operation: operation.to_string(),
        })
    }
}

pub(crate) fn validate_invoice_triggers(triggers: &InvoiceTriggerArgs) -> Result<()> {
    if triggers.amount_paid.is_some() && !triggers.paid {
        return Err(KobanError::InvalidPayload {
            message: "--amount-paid requires --paid".to_string(),
        });
    }
    Ok(())
}

pub(crate) fn validate_path_segment(label: &str, value: &str) -> Result<()> {
    let is_safe = !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'));

    if is_safe {
        Ok(())
    } else {
        Err(KobanError::InvalidPayload {
            message: format!("{label} must be a safe single path segment"),
        })
    }
}

pub(crate) fn invoice_payload(
    args: InvoicePayloadArgs,
    require_payload: bool,
    allow_empty_for_trigger: bool,
) -> Result<Value> {
    let has_raw = args.data.is_some() || args.data_file.is_some() || args.stdin;
    let has_guided = args.has_guided_fields();

    if has_raw && has_guided {
        return Err(KobanError::InvalidPayload {
            message: "raw JSON input cannot be combined with guided invoice flags".to_string(),
        });
    }

    if let Some(data) = args.data {
        return parse_json_payload(&data);
    }
    if let Some(path) = args.data_file {
        let data = fs::read_to_string(&path).map_err(|source| KobanError::InvalidPayload {
            message: format!("could not read {}: {source}", path.display()),
        })?;
        return parse_json_payload(&data);
    }
    if args.stdin {
        let mut data = String::new();
        io::stdin()
            .read_to_string(&mut data)
            .map_err(|source| KobanError::InvalidPayload {
                message: format!("could not read standard input: {source}"),
            })?;
        return parse_json_payload(&data);
    }

    if has_guided {
        return guided_invoice_payload(args);
    }

    if allow_empty_for_trigger {
        return Ok(json!({}));
    }

    if require_payload {
        Err(KobanError::InvalidPayload {
            message: "create requires JSON input or guided invoice flags".to_string(),
        })
    } else {
        Err(KobanError::InvalidPayload {
            message: "update requires JSON input, guided invoice flags, or a trigger flag"
                .to_string(),
        })
    }
}

pub(crate) fn parse_json_payload(data: &str) -> Result<Value> {
    let value =
        serde_json::from_str::<Value>(data).map_err(|source| KobanError::InvalidPayload {
            message: format!("JSON could not be parsed: {source}"),
        })?;
    if !value.is_object() {
        return Err(KobanError::InvalidPayload {
            message: "invoice payload must be a JSON object".to_string(),
        });
    }
    Ok(value)
}

pub(crate) fn guided_invoice_payload(args: InvoicePayloadArgs) -> Result<Value> {
    let mut body = serde_json::Map::new();
    insert_string(&mut body, "client_id", args.client_id);
    insert_string(&mut body, "date", args.date);
    insert_string(&mut body, "due_date", args.due_date);
    insert_string(&mut body, "number", args.number);
    insert_string(&mut body, "po_number", args.po_number);
    insert_string(&mut body, "public_notes", args.public_notes);
    insert_string(&mut body, "private_notes", args.private_notes);
    insert_string(&mut body, "terms", args.terms);
    insert_string(&mut body, "footer", args.footer);
    insert_string(&mut body, "project_id", args.project_id);

    if !args.line_items.is_empty() {
        let line_items = args
            .line_items
            .into_iter()
            .map(|line_item| parse_line_item(&line_item))
            .collect::<Result<Vec<_>>>()?;
        body.insert("line_items".to_string(), Value::Array(line_items));
    }

    Ok(Value::Object(body))
}

fn insert_string(map: &mut serde_json::Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(value) = value {
        map.insert(key.to_string(), Value::String(value));
    }
}

pub(crate) fn parse_line_item(input: &str) -> Result<Value> {
    let mut item = serde_json::Map::new();
    for part in input.split(',') {
        let Some((key, value)) = part.split_once('=') else {
            return Err(KobanError::InvalidPayload {
                message: format!("line item part `{part}` must use key=value"),
            });
        };
        let key = key.trim();
        if key.is_empty() {
            return Err(KobanError::InvalidPayload {
                message: format!("line item part `{part}` has an empty key"),
            });
        }
        item.insert(key.to_string(), parse_scalar(value.trim()));
    }

    if item.is_empty() {
        return Err(KobanError::InvalidPayload {
            message: "line item cannot be empty".to_string(),
        });
    }

    Ok(Value::Object(item))
}

pub(crate) fn parse_scalar(value: &str) -> Value {
    if value.eq_ignore_ascii_case("true") {
        Value::Bool(true)
    } else if value.eq_ignore_ascii_case("false") {
        Value::Bool(false)
    } else if value.eq_ignore_ascii_case("null") {
        Value::Null
    } else if let Ok(number) = value.parse::<serde_json::Number>() {
        Value::Number(number)
    } else {
        Value::String(value.to_string())
    }
}

impl InvoicePayloadArgs {
    pub(crate) fn has_guided_fields(&self) -> bool {
        self.client_id.is_some()
            || self.date.is_some()
            || self.due_date.is_some()
            || self.number.is_some()
            || self.po_number.is_some()
            || self.public_notes.is_some()
            || self.private_notes.is_some()
            || self.terms.is_some()
            || self.footer.is_some()
            || self.project_id.is_some()
            || !self.line_items.is_empty()
    }
}

impl InvoiceTriggerArgs {
    pub(crate) fn has_any(&self) -> bool {
        self.send_email
            || self.mark_sent
            || self.paid
            || self.amount_paid.is_some()
            || self.cancel
            || self.save_default_footer
            || self.save_default_terms
            || self.retry_e_send
    }

    pub(crate) fn requires_confirmation(&self) -> bool {
        self.send_email
            || self.paid
            || self.amount_paid.is_some()
            || self.cancel
            || self.retry_e_send
    }
}

pub(crate) fn push_invoice_triggers(
    query: &mut Vec<(String, String)>,
    triggers: &InvoiceTriggerArgs,
) {
    push_bool_query(query, "send_email", triggers.send_email);
    push_bool_query(query, "mark_sent", triggers.mark_sent);
    push_bool_query(query, "paid", triggers.paid);
    if let Some(amount_paid) = &triggers.amount_paid {
        query.push(("amount_paid".to_string(), amount_paid.clone()));
    }
    push_bool_query(query, "cancel", triggers.cancel);
    push_bool_query(query, "save_default_footer", triggers.save_default_footer);
    push_bool_query(query, "save_default_terms", triggers.save_default_terms);
    push_bool_query(query, "retry_e_send", triggers.retry_e_send);
}

fn push_bool_query(query: &mut Vec<(String, String)>, key: &str, enabled: bool) {
    if enabled {
        query.push((key.to_string(), "true".to_string()));
    }
}

pub(crate) fn render_dry_run(
    method: &str,
    path: &str,
    query: &[(String, String)],
    body: Option<&Value>,
    files: Option<&[PathBuf]>,
) -> Result<String> {
    let value = json!({
        "dry_run": true,
        "method": method,
        "path": path,
        "query": query.iter().map(|(key, value)| json!({"key": key, "value": value})).collect::<Vec<_>>(),
        "body": body,
        "files": files.map(|files| files.iter().map(|file| file.display().to_string()).collect::<Vec<_>>()),
    });
    serde_json::to_string_pretty(&value).map_err(|source| KobanError::Decode {
        message: source.to_string(),
    })
}
