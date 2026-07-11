wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "api",
});

use crate::exports::mapper::plugin::mapper::Guest;
use serde_json::{Map, Value, json};

struct MiniMaxMapper;

impl Guest for MiniMaxMapper {
    fn map_request(input: String) -> String {
        let mut request: Value = match serde_json::from_str(&input) {
            Ok(value) => value,
            Err(_) => return input,
        };
        let Some(object) = request.as_object_mut() else {
            return input;
        };

        object.insert("reasoning_split".into(), Value::Bool(true));
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
        map_thinking(object, &model);
        map_history(object);

        let has_tools = object
            .get("tools")
            .and_then(Value::as_array)
            .is_some_and(|tools| !tools.is_empty());
        if !has_tools {
            object.remove("tools");
            object.remove("tool_choice");
        }

        if object.get("stream").and_then(Value::as_bool) == Some(true) {
            object
                .entry("stream_options")
                .or_insert_with(|| json!({"include_usage": true}));
        }

        for unsupported in [
            "frequency_penalty",
            "presence_penalty",
            "logprobs",
            "top_logprobs",
            "n",
        ] {
            object.remove(unsupported);
        }
        object.retain(|_, value| !value.is_null());

        serde_json::to_string(&request).unwrap_or(input)
    }

    fn map_response(input: String) -> String {
        let mut response: Value = match serde_json::from_str(&input) {
            Ok(value) => value,
            Err(_) => return input,
        };
        normalize_response_choices(&mut response);
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
        normalize_stream_choices(&mut chunk);
        match serde_json::to_string(&chunk) {
            Ok(mapped) => format!("data: {mapped}"),
            Err(_) => line,
        }
    }
}

fn map_thinking(object: &mut Map<String, Value>, model: &str) {
    let thinking = object.remove("thinking");
    if model != "MiniMax-M3" {
        return;
    }

    let Some(mut thinking) = thinking else {
        return;
    };
    if thinking.get("type").and_then(Value::as_str) == Some("enabled")
        && let Some(thinking) = thinking.as_object_mut()
    {
        thinking.insert("type".into(), json!("adaptive"));
    }
    object.insert("thinking".into(), thinking);
}

fn map_history(object: &mut Map<String, Value>) {
    let Some(messages) = object.get_mut("messages").and_then(Value::as_array_mut) else {
        return;
    };

    for message in messages {
        let Some(message) = message.as_object_mut() else {
            continue;
        };
        if message.get("role").and_then(Value::as_str) != Some("assistant") {
            continue;
        }
        let Some(reasoning) = message
            .get("reasoning_content")
            .and_then(Value::as_str)
            .filter(|reasoning| !reasoning.is_empty())
            .map(str::to_string)
        else {
            continue;
        };
        message
            .entry("reasoning_details")
            .or_insert_with(|| json!([{"text": reasoning}]));
    }
}

fn normalize_response_choices(response: &mut Value) {
    let Some(choices) = response.get_mut("choices").and_then(Value::as_array_mut) else {
        return;
    };

    for choice in choices {
        let Some(message) = choice.get_mut("message").and_then(Value::as_object_mut) else {
            continue;
        };
        if message
            .get("reasoning_content")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
            && let Some(reasoning) = reasoning_details_text(message.get("reasoning_details"))
        {
            message.insert("reasoning_content".into(), Value::String(reasoning));
        }
        normalize_tool_indexes(message);
    }
}

fn normalize_stream_choices(chunk: &mut Value) {
    let Some(choices) = chunk.get_mut("choices").and_then(Value::as_array_mut) else {
        return;
    };

    for choice in choices {
        let Some(delta) = choice.get_mut("delta").and_then(Value::as_object_mut) else {
            continue;
        };

        if let Some(content) = delta.remove("content") {
            delta.insert("content_snapshot".into(), content);
        }

        let reasoning = reasoning_details_text(delta.get("reasoning_details"))
            .map(Value::String)
            .or_else(|| delta.remove("reasoning_content"));
        if let Some(reasoning) = reasoning {
            delta.insert("reasoning_content_snapshot".into(), reasoning);
        }
        normalize_tool_indexes(delta);
    }
}

fn reasoning_details_text(details: Option<&Value>) -> Option<String> {
    let text = details?
        .as_array()?
        .iter()
        .filter_map(|detail| detail.get("text").and_then(Value::as_str))
        .collect::<String>();
    (!text.is_empty()).then_some(text)
}

fn normalize_tool_indexes(payload: &mut Map<String, Value>) {
    let Some(tool_calls) = payload.get_mut("tool_calls").and_then(Value::as_array_mut) else {
        return;
    };
    for (index, tool_call) in tool_calls.iter_mut().enumerate() {
        if let Some(tool_call) = tool_call.as_object_mut() {
            tool_call.entry("index").or_insert(json!(index));
        }
    }
}

export!(MiniMaxMapper);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_m3_request_and_preserves_reasoning_history() {
        let mapped = MiniMaxMapper::map_request(
            json!({
                "model": "MiniMax-M3",
                "messages": [{
                    "role": "assistant",
                    "content": "答案",
                    "reasoning_content": "思考"
                }],
                "thinking": {"type": "enabled"},
                "max_tokens": 1024,
                "stream": true
            })
            .to_string(),
        );
        let mapped: Value = serde_json::from_str(&mapped).unwrap();

        assert_eq!(mapped["thinking"]["type"], "adaptive");
        assert_eq!(mapped["max_completion_tokens"], 1024);
        assert_eq!(mapped["reasoning_split"], true);
        assert_eq!(
            mapped["messages"][0]["reasoning_details"][0]["text"],
            "思考"
        );
        assert_eq!(mapped["stream_options"]["include_usage"], true);
        assert!(mapped.get("max_tokens").is_none());
    }

    #[test]
    fn marks_cumulative_stream_values_as_snapshots() {
        let mapped = MiniMaxMapper::map_stream_line(format!(
            "data: {}",
            json!({
                "id": "chunk-1",
                "object": "chat.completion.chunk",
                "created": 0,
                "model": "MiniMax-M3",
                "choices": [{
                    "index": 0,
                    "delta": {
                        "content": "你好",
                        "reasoning_details": [{"text": "想好了"}]
                    },
                    "finish_reason": null
                }]
            })
        ));
        let mapped: Value = serde_json::from_str(mapped.trim_start_matches("data: ")).unwrap();
        let delta = &mapped["choices"][0]["delta"];

        assert_eq!(delta["content_snapshot"], "你好");
        assert_eq!(delta["reasoning_content_snapshot"], "想好了");
        assert!(delta.get("content").is_none());
    }
}
