pub fn parse_name_value(v: Option<serde_json::Value>) -> Vec<(String, String)> {
    v.and_then(|v| v.as_array().cloned())
        .map(|arr| {
            arr.into_iter()
                .filter_map(|item| {
                    let name = item.get("name")?.as_str()?.to_string();
                    let value = item.get("value")?.as_str()?.to_string();
                    Some((name, value))
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn serialize_name_value(fields: &[(String, String)]) -> serde_json::Value {
    fields
        .iter()
        .map(|(n, v)| serde_json::json!({"name": n, "value": v}))
        .collect()
}
