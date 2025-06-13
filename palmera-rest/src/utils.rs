use sea_query::Value as SeaValue;
use serde_json::Value as JsonValue;

pub fn json_to_sea(value: &JsonValue) -> SeaValue {
    match value {
        JsonValue::Null => SeaValue::Bool(None).as_null(),
        JsonValue::Bool(b) => SeaValue::Bool(Some(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                SeaValue::BigInt(Some(i))
            } else if let Some(f) = n.as_f64() {
                SeaValue::Double(Some(f))
            } else {
                // Fallback for very large numbers
                SeaValue::String(Some(Box::new(n.to_string())))
            }
        }
        JsonValue::String(s) => SeaValue::String(Some(Box::new(s.to_string()))),
        JsonValue::Array(_) | JsonValue::Object(_) => SeaValue::Json(Some(Box::new(value.clone()))),
    }
}
