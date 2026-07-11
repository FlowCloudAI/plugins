wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "api",
});

use crate::exports::mapper::plugin::mapper::Guest;
use serde_json::{Value, json};

struct VolcengineMapper;

impl Guest for VolcengineMapper {
    fn map_request(input: String) -> String {
        let mut req: Value = match serde_json::from_str(&input) {
            Ok(value) => value,
            Err(_) => return input,
        };
        let Some(obj) = req.as_object_mut() else {
            return input;
        };

        if obj
            .get("tools")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
        {
            obj.remove("tools");
            obj.remove("tool_choice");
        }

        if let Some(effort) = obj.remove("thinking_effort") {
            let effort = match effort.as_str() {
                Some("low") => Some("low"),
                Some("medium") => Some("medium"),
                Some("high" | "xhigh") => Some("high"),
                _ => None,
            };
            if let Some(effort) = effort {
                obj.insert("reasoning_effort".into(), json!(effort));
            }
        }

        if obj.get("stream").and_then(Value::as_bool) == Some(true)
            && !obj.contains_key("stream_options")
        {
            obj.insert("stream_options".into(), json!({"include_usage": true}));
        }

        obj.retain(|_, value| !value.is_null());
        serde_json::to_string(&req).unwrap_or(input)
    }

    fn map_response(input: String) -> String {
        let mut response: Value = match serde_json::from_str(&input) {
            Ok(value) => value,
            Err(_) => return input,
        };
        normalize_choices(&mut response, "message");
        serde_json::to_string(&response).unwrap_or(input)
    }

    fn map_stream_line(line: String) -> String {
        let trimmed = line.trim();
        let Some(json_str) = trimmed.strip_prefix("data:").map(str::trim) else {
            return line;
        };
        if json_str.is_empty() || json_str == "[DONE]" {
            return line;
        }

        let mut chunk: Value = match serde_json::from_str(json_str) {
            Ok(value) => value,
            Err(_) => return line,
        };
        normalize_choices(&mut chunk, "delta");
        match serde_json::to_string(&chunk) {
            Ok(mapped) => format!("data: {mapped}"),
            Err(_) => line,
        }
    }
}

fn normalize_choices(value: &mut Value, payload_key: &str) {
    let Some(choices) = value.get_mut("choices").and_then(Value::as_array_mut) else {
        return;
    };

    for choice in choices {
        let Some(payload) = choice.get_mut(payload_key).and_then(Value::as_object_mut) else {
            continue;
        };
        payload.entry("reasoning_content").or_insert(Value::Null);

        if let Some(tool_calls) = payload.get_mut("tool_calls").and_then(Value::as_array_mut) {
            for (index, tool_call) in tool_calls.iter_mut().enumerate() {
                if let Some(tool_call) = tool_call.as_object_mut() {
                    tool_call.entry("index").or_insert(json!(index));
                }
            }
        }
    }
}

export!(VolcengineMapper);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_request_and_stream_fields() {
        let mapped = VolcengineMapper::map_request(
            json!({
                "model": "doubao-seed-2-1-pro-260628",
                "messages": [{"role": "user", "content": "你好"}],
                "thinking_effort": "medium",
                "tools": [],
                "stream": true
            })
            .to_string(),
        );
        let mapped: Value = serde_json::from_str(&mapped).unwrap();

        assert_eq!(mapped["reasoning_effort"], "medium");
        assert_eq!(mapped["stream_options"]["include_usage"], true);
        assert!(mapped.get("thinking_effort").is_none());
        assert!(mapped.get("tools").is_none());
    }
}
