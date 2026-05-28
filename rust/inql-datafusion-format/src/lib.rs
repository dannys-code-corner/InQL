use std::sync::Arc;

use crc32fast::Hasher as Crc32Hasher;
use datafusion::arrow::array::{
    Array, ArrayRef, BooleanArray, Int64Array, LargeStringArray, StringArray, StringViewArray,
};
use datafusion::arrow::datatypes::DataType;
use datafusion::common::{DataFusionError, Result, ScalarValue};
use datafusion::execution::context::SessionContext;
use datafusion::logical_expr::{ColumnarValue, Volatility, create_udf};
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use serde_json::{Map, Value};
use sha1::{Digest, Sha1};
use url::Url;
use xxhash_rust::xxh64::xxh64;

const URL_COMPONENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}');

pub fn register_format_udfs(ctx: SessionContext) {
    for udf in [
        unary_string_udf("sha1", hash_sha1),
        unary_string_udf("crc32", hash_crc32),
        unary_string_udf("xxhash64", hash_xxhash64),
        unary_string_udf("url_encode", encode_url_component),
        unary_string_udf("url_decode", decode_url_component),
        unary_string_udf("try_url_decode", try_decode_url_component),
        binary_string_udf("parse_url", parse_url_part),
        unary_string_udf("parse_json", parse_json),
        unary_string_udf("check_json", check_json),
        unary_string_udf("schema_of_json", schema_of_json),
        unary_string_udf("json_array_length", json_array_length),
        unary_string_udf("json_object_keys", json_object_keys),
        binary_string_udf("get_json_object", get_json_object),
        binary_string_udf("json_extract_path_text", json_extract_path_text),
        binary_string_udf("from_json", from_json),
        binary_string_udf("try_from_json", try_from_json),
        unary_string_udf("to_json", to_json),
        unary_string_udf("schema_of_csv", schema_of_csv),
        binary_string_udf("from_csv", from_csv),
        unary_string_udf("to_csv", to_csv),
    ] {
        ctx.register_udf(udf);
    }
}

fn unary_string_udf(
    name: &'static str,
    fun: fn(Option<&str>) -> Result<ScalarValue>,
) -> datafusion::logical_expr::ScalarUDF {
    create_udf(
        name,
        vec![DataType::Utf8],
        return_type_for(name),
        Volatility::Immutable,
        Arc::new(move |args| apply_unary_string(args, fun)),
    )
}

fn binary_string_udf(
    name: &'static str,
    fun: fn(Option<&str>, Option<&str>) -> Result<ScalarValue>,
) -> datafusion::logical_expr::ScalarUDF {
    create_udf(
        name,
        vec![DataType::Utf8, DataType::Utf8],
        DataType::Utf8,
        Volatility::Immutable,
        Arc::new(move |args| apply_binary_string(args, fun)),
    )
}

fn return_type_for(name: &str) -> DataType {
    match name {
        "check_json" => DataType::Boolean,
        "json_array_length" => DataType::Int64,
        _ => DataType::Utf8,
    }
}

fn apply_unary_string(
    args: &[ColumnarValue],
    fun: fn(Option<&str>) -> Result<ScalarValue>,
) -> Result<ColumnarValue> {
    require_arity(args, 1)?;
    if all_scalar(args) {
        let value = scalar_string(args[0].clone())?;
        return fun(value.as_deref()).map(ColumnarValue::Scalar);
    }

    let len = row_count(args);
    let mut string_values: Vec<Option<String>> = Vec::with_capacity(len);
    let mut bool_values: Vec<Option<bool>> = Vec::with_capacity(len);
    let mut int_values: Vec<Option<i64>> = Vec::with_capacity(len);
    let mut result_kind: Option<&'static str> = None;
    for idx in 0..len {
        let value = string_at(&args[0], idx)?;
        let scalar = fun(value.as_deref())?;
        match scalar {
            ScalarValue::Utf8(item) => {
                result_kind = Some("string");
                string_values.push(item);
            }
            ScalarValue::Boolean(item) => {
                result_kind = Some("bool");
                bool_values.push(item);
            }
            ScalarValue::Int64(item) => {
                result_kind = Some("int");
                int_values.push(item);
            }
            _ => return internal_error("unexpected format UDF return type"),
        }
    }
    array_result(
        result_kind.unwrap_or("string"),
        string_values,
        bool_values,
        int_values,
    )
}

fn apply_binary_string(
    args: &[ColumnarValue],
    fun: fn(Option<&str>, Option<&str>) -> Result<ScalarValue>,
) -> Result<ColumnarValue> {
    require_arity(args, 2)?;
    if all_scalar(args) {
        let left = scalar_string(args[0].clone())?;
        let right = scalar_string(args[1].clone())?;
        return fun(left.as_deref(), right.as_deref()).map(ColumnarValue::Scalar);
    }

    let len = row_count(args);
    let mut values: Vec<Option<String>> = Vec::with_capacity(len);
    for idx in 0..len {
        let left = string_at(&args[0], idx)?;
        let right = string_at(&args[1], idx)?;
        match fun(left.as_deref(), right.as_deref())? {
            ScalarValue::Utf8(item) => values.push(item),
            _ => return internal_error("unexpected binary format UDF return type"),
        }
    }
    Ok(ColumnarValue::Array(Arc::new(StringArray::from(values))))
}

fn require_arity(args: &[ColumnarValue], expected: usize) -> Result<()> {
    if args.len() == expected {
        return Ok(());
    }
    internal_error(format!(
        "format UDF expected {expected} arguments, got {}",
        args.len()
    ))
}

fn all_scalar(args: &[ColumnarValue]) -> bool {
    args.iter()
        .all(|arg| matches!(arg, ColumnarValue::Scalar(_)))
}

fn row_count(args: &[ColumnarValue]) -> usize {
    args.iter()
        .find_map(|arg| match arg {
            ColumnarValue::Array(array) => Some(array.len()),
            ColumnarValue::Scalar(_) => None,
        })
        .unwrap_or(1)
}

fn array_result(
    kind: &str,
    string_values: Vec<Option<String>>,
    bool_values: Vec<Option<bool>>,
    int_values: Vec<Option<i64>>,
) -> Result<ColumnarValue> {
    match kind {
        "bool" => Ok(ColumnarValue::Array(Arc::new(BooleanArray::from(
            bool_values,
        )))),
        "int" => Ok(ColumnarValue::Array(Arc::new(Int64Array::from(int_values)))),
        _ => Ok(ColumnarValue::Array(Arc::new(StringArray::from(
            string_values,
        )))),
    }
}

fn scalar_string(arg: ColumnarValue) -> Result<Option<String>> {
    match arg {
        ColumnarValue::Scalar(ScalarValue::Utf8(value)) => Ok(value),
        ColumnarValue::Scalar(ScalarValue::LargeUtf8(value)) => Ok(value),
        ColumnarValue::Scalar(ScalarValue::Utf8View(value)) => Ok(value),
        ColumnarValue::Scalar(ScalarValue::Null) => Ok(None),
        ColumnarValue::Scalar(value) => Ok(Some(value.to_string())),
        ColumnarValue::Array(array) => string_at(&ColumnarValue::Array(array), 0),
    }
}

fn string_at(arg: &ColumnarValue, idx: usize) -> Result<Option<String>> {
    match arg {
        ColumnarValue::Scalar(value) => scalar_string(ColumnarValue::Scalar(value.clone())),
        ColumnarValue::Array(array) => string_array_value(array, idx),
    }
}

fn string_array_value(array: &ArrayRef, idx: usize) -> Result<Option<String>> {
    if array.is_null(idx) {
        return Ok(None);
    }
    match array.data_type() {
        DataType::Utf8 => {
            let strings = array
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    DataFusionError::Internal(
                        "format UDF could not downcast Utf8 array".to_string(),
                    )
                })?;
            Ok(Some(strings.value(idx).to_string()))
        }
        DataType::LargeUtf8 => {
            let strings = array
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .ok_or_else(|| {
                    DataFusionError::Internal(
                        "format UDF could not downcast LargeUtf8 array".to_string(),
                    )
                })?;
            Ok(Some(strings.value(idx).to_string()))
        }
        DataType::Utf8View => {
            let strings = array
                .as_any()
                .downcast_ref::<StringViewArray>()
                .ok_or_else(|| {
                    DataFusionError::Internal(
                        "format UDF could not downcast Utf8View array".to_string(),
                    )
                })?;
            Ok(Some(strings.value(idx).to_string()))
        }
        _ => Ok(Some(format!("{:?}", array))),
    }
}

fn hash_sha1(value: Option<&str>) -> Result<ScalarValue> {
    match value {
        Some(text) => {
            let mut hasher = Sha1::new();
            hasher.update(text.as_bytes());
            Ok(ScalarValue::Utf8(Some(hex::encode(hasher.finalize()))))
        }
        None => Ok(ScalarValue::Utf8(None)),
    }
}

fn hash_crc32(value: Option<&str>) -> Result<ScalarValue> {
    match value {
        Some(text) => {
            let mut hasher = Crc32Hasher::new();
            hasher.update(text.as_bytes());
            Ok(ScalarValue::Utf8(Some(format!(
                "{:08x}",
                hasher.finalize()
            ))))
        }
        None => Ok(ScalarValue::Utf8(None)),
    }
}

fn hash_xxhash64(value: Option<&str>) -> Result<ScalarValue> {
    match value {
        Some(text) => Ok(ScalarValue::Utf8(Some(format!(
            "{:016x}",
            xxh64(text.as_bytes(), 0)
        )))),
        None => Ok(ScalarValue::Utf8(None)),
    }
}

fn encode_url_component(value: Option<&str>) -> Result<ScalarValue> {
    Ok(ScalarValue::Utf8(value.map(|text| {
        utf8_percent_encode(text, URL_COMPONENT_ENCODE_SET).to_string()
    })))
}

fn decode_url_component(value: Option<&str>) -> Result<ScalarValue> {
    if value.is_none() {
        return Ok(ScalarValue::Utf8(None));
    }
    match try_decode_url_text(value)? {
        Some(text) => Ok(ScalarValue::Utf8(Some(text))),
        None => internal_error("invalid percent-encoded URL component"),
    }
}

fn try_decode_url_component(value: Option<&str>) -> Result<ScalarValue> {
    Ok(ScalarValue::Utf8(try_decode_url_text(value)?))
}

fn try_decode_url_text(value: Option<&str>) -> Result<Option<String>> {
    match value {
        Some(text) => {
            if !percent_escapes_are_valid(text) {
                return Ok(None);
            }
            match percent_decode_str(text).decode_utf8() {
                Ok(decoded) => Ok(Some(decoded.to_string())),
                Err(_) => Ok(None),
            }
        }
        None => Ok(None),
    }
}

fn percent_escapes_are_valid(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut idx = 0;
    while idx < bytes.len() {
        if bytes[idx] == b'%' {
            if idx + 2 >= bytes.len()
                || !bytes[idx + 1].is_ascii_hexdigit()
                || !bytes[idx + 2].is_ascii_hexdigit()
            {
                return false;
            }
            idx += 3;
        } else {
            idx += 1;
        }
    }
    true
}

fn parse_url_part(url: Option<&str>, part: Option<&str>) -> Result<ScalarValue> {
    let Some(raw_url) = url else {
        return Ok(ScalarValue::Utf8(None));
    };
    let Some(raw_part) = part else {
        return Ok(ScalarValue::Utf8(None));
    };
    let parsed = Url::parse(raw_url).map_err(|err| DataFusionError::Execution(err.to_string()))?;
    let value = match raw_part.to_ascii_lowercase().as_str() {
        "scheme" => Some(parsed.scheme().to_string()),
        "host" => parsed.host_str().map(str::to_string),
        "path" => Some(parsed.path().to_string()),
        "query" => parsed.query().map(str::to_string),
        "fragment" => parsed.fragment().map(str::to_string),
        "port" => parsed.port().map(|port| port.to_string()),
        "username" => Some(parsed.username().to_string()),
        "password" => parsed.password().map(str::to_string),
        other => return internal_error(format!("unknown parse_url part `{other}`")),
    };
    Ok(ScalarValue::Utf8(value))
}

fn parse_json(value: Option<&str>) -> Result<ScalarValue> {
    match value {
        Some(text) => Ok(ScalarValue::Utf8(Some(normalized_json(text)?))),
        None => Ok(ScalarValue::Utf8(None)),
    }
}

fn check_json(value: Option<&str>) -> Result<ScalarValue> {
    match value {
        Some(text) => Ok(ScalarValue::Boolean(Some(
            serde_json::from_str::<Value>(text).is_ok(),
        ))),
        None => Ok(ScalarValue::Boolean(None)),
    }
}

fn schema_of_json(value: Option<&str>) -> Result<ScalarValue> {
    match value {
        Some(text) => {
            let parsed = parse_json_value(text)?;
            Ok(ScalarValue::Utf8(Some(json_schema(&parsed))))
        }
        None => Ok(ScalarValue::Utf8(None)),
    }
}

fn json_array_length(value: Option<&str>) -> Result<ScalarValue> {
    match value {
        Some(text) => {
            let parsed = parse_json_value(text)?;
            let length = parsed.as_array().map(|items| items.len() as i64);
            Ok(ScalarValue::Int64(length))
        }
        None => Ok(ScalarValue::Int64(None)),
    }
}

fn json_object_keys(value: Option<&str>) -> Result<ScalarValue> {
    match value {
        Some(text) => {
            let parsed = parse_json_value(text)?;
            let keys: Vec<Value> = parsed
                .as_object()
                .map(|object| object.keys().cloned().map(Value::String).collect())
                .unwrap_or_default();
            Ok(ScalarValue::Utf8(Some(Value::Array(keys).to_string())))
        }
        None => Ok(ScalarValue::Utf8(None)),
    }
}

fn get_json_object(value: Option<&str>, path: Option<&str>) -> Result<ScalarValue> {
    json_path_value(value, path, false)
}

fn json_extract_path_text(value: Option<&str>, path: Option<&str>) -> Result<ScalarValue> {
    json_path_value(value, path, true)
}

fn json_path_value(
    value: Option<&str>,
    path: Option<&str>,
    text_mode: bool,
) -> Result<ScalarValue> {
    let Some(raw_json) = value else {
        return Ok(ScalarValue::Utf8(None));
    };
    let Some(raw_path) = path else {
        return Ok(ScalarValue::Utf8(None));
    };
    let parsed = parse_json_value(raw_json)?;
    let Some(selected) = select_json_path(&parsed, raw_path) else {
        return Ok(ScalarValue::Utf8(None));
    };
    if text_mode {
        if let Some(text) = selected.as_str() {
            return Ok(ScalarValue::Utf8(Some(text.to_string())));
        }
    }
    Ok(ScalarValue::Utf8(Some(selected.to_string())))
}

fn from_json(value: Option<&str>, _schema: Option<&str>) -> Result<ScalarValue> {
    parse_json(value)
}

fn try_from_json(value: Option<&str>, _schema: Option<&str>) -> Result<ScalarValue> {
    match value {
        Some(text) => match normalized_json(text) {
            Ok(parsed) => Ok(ScalarValue::Utf8(Some(parsed))),
            Err(_) => Ok(ScalarValue::Utf8(None)),
        },
        None => Ok(ScalarValue::Utf8(None)),
    }
}

fn to_json(value: Option<&str>) -> Result<ScalarValue> {
    Ok(ScalarValue::Utf8(
        value.map(|text| Value::String(text.to_string()).to_string()),
    ))
}

fn normalized_json(text: &str) -> Result<String> {
    Ok(parse_json_value(text)?.to_string())
}

fn parse_json_value(text: &str) -> Result<Value> {
    serde_json::from_str::<Value>(text).map_err(|err| DataFusionError::Execution(err.to_string()))
}

fn select_json_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    if path == "$" || path.is_empty() {
        return Some(value);
    }
    let mut current = value;
    let path = path
        .strip_prefix("$.")
        .or_else(|| path.strip_prefix('$'))
        .unwrap_or(path);
    for segment in path.split('.') {
        if segment.is_empty() {
            continue;
        }
        current = select_json_segment(current, segment)?;
    }
    Some(current)
}

fn select_json_segment<'a>(value: &'a Value, segment: &str) -> Option<&'a Value> {
    if let Some((field, index_part)) = segment.split_once('[') {
        let nested = if field.is_empty() {
            value
        } else {
            value.as_object()?.get(field)?
        };
        let index_text = index_part.strip_suffix(']')?;
        let index: usize = index_text.parse().ok()?;
        return nested.as_array()?.get(index);
    }
    value.as_object()?.get(segment)
}

fn json_schema(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_string(),
        Value::Bool(_) => "BOOLEAN".to_string(),
        Value::Number(number) => {
            if number.is_i64() || number.is_u64() {
                "BIGINT".to_string()
            } else {
                "DOUBLE".to_string()
            }
        }
        Value::String(_) => "STRING".to_string(),
        Value::Array(values) => {
            let item_schema = values
                .first()
                .map(json_schema)
                .unwrap_or_else(|| "UNKNOWN".to_string());
            format!("ARRAY<{item_schema}>")
        }
        Value::Object(object) => {
            let fields = object
                .iter()
                .map(|(name, item)| format!("{name}: {}", json_schema(item)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("STRUCT<{fields}>")
        }
    }
}

fn schema_of_csv(value: Option<&str>) -> Result<ScalarValue> {
    let Some(text) = value else {
        return Ok(ScalarValue::Utf8(None));
    };
    let record = first_csv_record(text)?;
    let fields = record
        .iter()
        .enumerate()
        .map(|(idx, item)| format!("_c{idx}: {}", scalar_schema(item)))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(ScalarValue::Utf8(Some(format!("STRUCT<{fields}>"))))
}

fn from_csv(value: Option<&str>, schema: Option<&str>) -> Result<ScalarValue> {
    let Some(text) = value else {
        return Ok(ScalarValue::Utf8(None));
    };
    let record = first_csv_record(text)?;
    let names = csv_schema_field_names(schema.unwrap_or(""));
    if names.is_empty() {
        let values = record
            .iter()
            .map(|field| Value::String(field.to_string()))
            .collect();
        return Ok(ScalarValue::Utf8(Some(Value::Array(values).to_string())));
    }
    let mut object = Map::new();
    for (idx, name) in names.iter().enumerate() {
        let value = record.get(idx).unwrap_or("");
        object.insert(name.clone(), Value::String(value.to_string()));
    }
    Ok(ScalarValue::Utf8(Some(Value::Object(object).to_string())))
}

fn to_csv(value: Option<&str>) -> Result<ScalarValue> {
    let Some(text) = value else {
        return Ok(ScalarValue::Utf8(None));
    };
    let fields = match serde_json::from_str::<Value>(text) {
        Ok(Value::Array(values)) => values.into_iter().map(csv_json_value).collect(),
        Ok(Value::Object(object)) => object.into_values().map(csv_json_value).collect(),
        _ => vec![text.to_string()],
    };
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(vec![]);
    writer
        .write_record(fields)
        .map_err(|err| DataFusionError::Execution(err.to_string()))?;
    let bytes = writer
        .into_inner()
        .map_err(|err| DataFusionError::Execution(err.to_string()))?;
    let row =
        String::from_utf8(bytes).map_err(|err| DataFusionError::Execution(err.to_string()))?;
    Ok(ScalarValue::Utf8(Some(
        row.trim_end_matches(['\r', '\n']).to_string(),
    )))
}

fn first_csv_record(text: &str) -> Result<csv::StringRecord> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(text.as_bytes());
    match reader.records().next() {
        Some(record) => record.map_err(|err| DataFusionError::Execution(err.to_string())),
        None => Ok(csv::StringRecord::new()),
    }
}

fn csv_schema_field_names(schema: &str) -> Vec<String> {
    schema
        .trim()
        .trim_start_matches("STRUCT<")
        .trim_end_matches('>')
        .split(',')
        .filter_map(|part| {
            let name = part.trim().split([':', ' ']).next().unwrap_or("").trim();
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        })
        .collect()
}

fn scalar_schema(value: &str) -> &'static str {
    if value.parse::<i64>().is_ok() {
        "BIGINT"
    } else if value.parse::<f64>().is_ok() {
        "DOUBLE"
    } else if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
        "BOOLEAN"
    } else {
        "STRING"
    }
}

fn csv_json_value(value: Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text,
        other => other.to_string(),
    }
}

fn internal_error<T>(message: impl Into<String>) -> Result<T> {
    Err(DataFusionError::Internal(message.into()))
}
