use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use serde_json::Value;

use crate::{KobanError, Result};

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
