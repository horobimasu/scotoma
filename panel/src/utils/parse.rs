use serde_json::Value;

pub fn get_str(val: &Value) -> String {
    val
        .as_str()
        .unwrap_or("")
        .to_string()
}

pub fn get_bool(val: &Value) -> bool {
    val
        .as_bool()
        .unwrap_or(false)
}

pub fn get_usize(val: &Value) -> usize {
    val
        .as_u64()
        .unwrap_or(0) as usize
}

pub fn get_str_array(val: &Value) -> Vec<String> {
    let joined = val
        .as_str()
        .unwrap_or("");

    if joined.is_empty() {
        return Vec::new();
    }

    joined
        .split("\n")
        .map(|str| str.to_string())
        .collect()
}
