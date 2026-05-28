use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use serde_json::Value;

use crate::{KobanError, Result, cli::ResourcePayloadArgs};

#[derive(Debug, Clone, Default)]
pub struct GenericPayloadArgs {
    pub data: Option<String>,
    pub data_file: Option<PathBuf>,
    pub stdin: bool,
    pub fields: Vec<String>,
}

pub(crate) fn generic_payload(args: GenericPayloadArgs, require_payload: bool) -> Result<Value> {
    let has_raw = args.data.is_some() || args.data_file.is_some() || args.stdin;
    let has_fields = !args.fields.is_empty();

    if has_raw && has_fields {
        return Err(KobanError::InvalidPayload {
            message: "raw JSON input cannot be combined with guided fields".to_string(),
        });
    }

    if let Some(data) = args.data {
        return parse_json_payload(&data, "payload");
    }
    if let Some(path) = args.data_file {
        let data = fs::read_to_string(&path).map_err(|source| KobanError::InvalidPayload {
            message: format!("could not read {}: {source}", path.display()),
        })?;
        return parse_json_payload(&data, "payload");
    }
    if args.stdin {
        let mut data = String::new();
        io::stdin()
            .read_to_string(&mut data)
            .map_err(|source| KobanError::InvalidPayload {
                message: format!("could not read standard input: {source}"),
            })?;
        return parse_json_payload(&data, "payload");
    }

    if has_fields {
        return object_from_fields(args.fields);
    }

    if require_payload {
        Err(KobanError::InvalidPayload {
            message: "write command requires JSON input or guided fields".to_string(),
        })
    } else {
        Ok(Value::Object(serde_json::Map::new()))
    }
}

pub(crate) fn resource_payload(args: ResourcePayloadArgs, require_payload: bool) -> Result<Value> {
    let has_raw = args.data.is_some() || args.data_file.is_some() || args.stdin;
    if has_raw && !args.line_items.is_empty() {
        return Err(KobanError::InvalidPayload {
            message: "raw JSON input cannot be combined with --line-item".to_string(),
        });
    }

    let mut fields = args.fields;
    push_optional_field(&mut fields, "name", args.name);
    push_optional_field(&mut fields, "number", args.number);
    push_optional_field(&mut fields, "client_id", args.client_id);
    push_optional_field(&mut fields, "vendor_id", args.vendor_id);
    push_optional_field(&mut fields, "project_id", args.project_id);
    push_optional_field(&mut fields, "date", args.date);
    push_optional_field(&mut fields, "due_date", args.due_date);
    push_optional_field(&mut fields, "amount", args.amount);
    push_optional_field(&mut fields, "price", args.price);
    push_optional_field(&mut fields, "quantity", args.quantity);
    push_optional_field(&mut fields, "public_notes", args.public_notes);
    push_optional_field(&mut fields, "private_notes", args.private_notes);

    let mut body = generic_payload(
        GenericPayloadArgs {
            data: args.data,
            data_file: args.data_file,
            stdin: args.stdin,
            fields,
        },
        require_payload && args.line_items.is_empty(),
    )?;

    if !args.line_items.is_empty() {
        let Some(map) = body.as_object_mut() else {
            return Err(KobanError::InvalidPayload {
                message: "payload must be a JSON object".to_string(),
            });
        };
        let line_items = args
            .line_items
            .into_iter()
            .map(|line_item| crate::invoice::parse_line_item(&line_item))
            .collect::<Result<Vec<_>>>()?;
        map.insert("line_items".to_string(), Value::Array(line_items));
    }

    Ok(body)
}

pub(crate) fn merge_resource_action_payload(target: &mut Value, extra: Value) {
    let Some(extra) = extra.as_object() else {
        return;
    };
    let Some(target) = target.as_object_mut() else {
        return;
    };
    for (key, value) in extra {
        if key != "action" && key != "ids" {
            target.insert(key.clone(), value.clone());
        }
    }
}

fn push_optional_field(fields: &mut Vec<String>, key: &str, value: Option<String>) {
    if let Some(value) = value {
        fields.push(format!("{key}={value}"));
    }
}

pub(crate) fn parse_json_payload(data: &str, label: &str) -> Result<Value> {
    let value =
        serde_json::from_str::<Value>(data).map_err(|source| KobanError::InvalidPayload {
            message: format!("{label} JSON could not be parsed: {source}"),
        })?;
    if !value.is_object() {
        return Err(KobanError::InvalidPayload {
            message: format!("{label} must be a JSON object"),
        });
    }
    Ok(value)
}

pub(crate) fn object_from_fields(fields: Vec<String>) -> Result<Value> {
    let mut body = serde_json::Map::new();
    for field in fields {
        let Some((key, value)) = field.split_once('=') else {
            return Err(KobanError::InvalidPayload {
                message: format!("field `{field}` must use key=value"),
            });
        };
        insert_path(&mut body, key.trim(), parse_scalar(value.trim()))?;
    }
    Ok(Value::Object(body))
}

pub(crate) fn insert_string(
    map: &mut serde_json::Map<String, Value>,
    key: &str,
    value: Option<String>,
) {
    if let Some(value) = value {
        map.insert(key.to_string(), Value::String(value));
    }
}

pub(crate) fn insert_path(
    map: &mut serde_json::Map<String, Value>,
    path: &str,
    value: Value,
) -> Result<()> {
    if path.is_empty() || path.split('.').any(str::is_empty) {
        return Err(KobanError::InvalidPayload {
            message: format!("field path `{path}` must not be empty"),
        });
    }

    let segments = path.split('.').collect::<Vec<_>>();
    insert_path_segments(map, &segments, value);
    Ok(())
}

fn insert_path_segments(map: &mut serde_json::Map<String, Value>, segments: &[&str], value: Value) {
    if segments.len() == 1 {
        map.insert(segments[0].to_string(), value);
        return;
    }

    let entry = map
        .entry(segments[0].to_string())
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    if !entry.is_object() {
        *entry = Value::Object(serde_json::Map::new());
    }
    insert_path_segments(
        entry.as_object_mut().expect("object inserted above"),
        &segments[1..],
        value,
    );
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
