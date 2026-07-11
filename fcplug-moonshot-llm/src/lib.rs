wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "api",
});

use crate::exports::mapper::plugin::mapper::Guest;
use serde_json::{Map, Value, json};

struct MoonshotMapper;

impl Guest for MoonshotMapper {
    fn map_request(input: String) -> String {
        let mut request: Value = match serde_json::from_str(&input) {
            Ok(value) => value,
            Err(_) => return input,
        };
        let Some(object) = request.as_object_mut() else {
            return input;
        };

        object.remove("thinking_effort");
        if !object.contains_key("max_completion_tokens")
            && let Some(max_tokens) = object.remove("max_tokens")
        {
            object.insert("max_completion_tokens".into(), max_tokens);
        }

        let model = object
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if model.starts_with("kimi-k2.") {
            map_k2_parameters(object, &model);
        } else {
            object.remove("thinking");
        }

        let has_tools = object
            .get("tools")
            .and_then(Value::as_array)
            .is_some_and(|tools| !tools.is_empty());
        if !has_tools {
            object.remove("tools");
            object.remove("tool_choice");
        } else if !matches!(
            object.get("tool_choice").and_then(Value::as_str),
            None | Some("auto" | "none")
        ) {
            object.insert("tool_choice".into(), json!("auto"));
        }

        if object.get("stream").and_then(Value::as_bool) == Some(true) {
            object
                .entry("stream_options")
                .or_insert_with(|| json!({"include_usage": true}));
        }

        object.remove("logprobs");
        object.remove("top_logprobs");
        object.retain(|_, value| !value.is_null());
        serde_json::to_string(&request).unwrap_or(input)
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
        lift_stream_usage(&mut chunk);
        normalize_choices(&mut chunk, "delta");
        match serde_json::to_string(&chunk) {
            Ok(mapped) => format!("data: {mapped}"),
            Err(_) => line,
        }
    }
}

fn map_k2_parameters(object: &mut Map<String, Value>, model: &str) {
    let is_k27 = model.starts_with("kimi-k2.7-");
    let thinking_disabled = !is_k27
        && object
            .get("thinking")
            .and_then(|thinking| thinking.get("type"))
            .and_then(Value::as_str)
            == Some("disabled");

    if is_k27 {
        object.insert("thinking".into(), json!({"type": "enabled", "keep": "all"}));
    }
    object.insert(
        "temperature".into(),
        json!(if thinking_disabled { 0.6 } else { 1.0 }),
    );
    object.insert("top_p".into(), json!(0.95));
    object.insert("n".into(), json!(1));
    object.insert("presence_penalty".into(), json!(0.0));
    object.insert("frequency_penalty".into(), json!(0.0));
}

fn normalize_choices(value: &mut Value, payload_key: &str) {
    let Some(choices) = value.get_mut("choices").and_then(Value::as_array_mut) else {
        return;
    };
    for choice in choices {
        let Some(payload) = choice.get_mut(payload_key).and_then(Value::as_object_mut) else {
            continue;
        };
        if payload_key == "message" {
            payload.entry("reasoning_content").or_insert(Value::Null);
        }
        if let Some(tool_calls) = payload.get_mut("tool_calls").and_then(Value::as_array_mut) {
            for (index, tool_call) in tool_calls.iter_mut().enumerate() {
                if let Some(tool_call) = tool_call.as_object_mut() {
                    tool_call.entry("index").or_insert(json!(index));
                }
            }
        }
    }
}

fn lift_stream_usage(chunk: &mut Value) {
    if chunk.get("usage").is_some_and(|usage| !usage.is_null()) {
        return;
    }
    let usage = chunk
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.iter().find_map(|choice| choice.get("usage")))
        .cloned();
    if let Some(usage) = usage
        && let Some(object) = chunk.as_object_mut()
    {
        object.insert("usage".into(), usage);
    }
}

export!(MoonshotMapper);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_k27_fixed_parameters_and_preserved_thinking() {
        let mapped = MoonshotMapper::map_request(
            json!({
                "model": "kimi-k2.7-code",
                "messages": [{"role": "user", "content": "写代码"}],
                "thinking": {"type": "disabled"},
                "temperature": 0.2,
                "max_tokens": 4096,
                "stream": true
            })
            .to_string(),
        );
        let mapped: Value = serde_json::from_str(&mapped).unwrap();

        assert_eq!(mapped["thinking"]["type"], "enabled");
        assert_eq!(mapped["thinking"]["keep"], "all");
        assert_eq!(mapped["temperature"], 1.0);
        assert_eq!(mapped["top_p"], 0.95);
        assert_eq!(mapped["max_completion_tokens"], 4096);
        assert_eq!(mapped["stream_options"]["include_usage"], true);
        assert!(mapped.get("max_tokens").is_none());
    }

    #[test]
    fn uses_non_thinking_temperature_for_k26() {
        let mapped = MoonshotMapper::map_request(
            json!({
                "model": "kimi-k2.6",
                "messages": [],
                "thinking": {"type": "disabled"}
            })
            .to_string(),
        );
        let mapped: Value = serde_json::from_str(&mapped).unwrap();

        assert_eq!(mapped["temperature"], 0.6);
        assert_eq!(mapped["thinking"]["type"], "disabled");
    }

    #[test]
    fn lifts_usage_from_final_stream_choice() {
        let mapped = MoonshotMapper::map_stream_line(format!(
            "data: {}",
            json!({
                "id": "chunk-1",
                "object": "chat.completion.chunk",
                "created": 0,
                "model": "kimi-k2.6",
                "choices": [{
                    "index": 0,
                    "delta": {},
                    "finish_reason": "stop",
                    "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
                }]
            })
        ));
        let mapped: Value = serde_json::from_str(mapped.trim_start_matches("data: ")).unwrap();

        assert_eq!(mapped["usage"]["total_tokens"], 15);
    }
}
