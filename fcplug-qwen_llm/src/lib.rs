wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "api",
});

use crate::exports::mapper::plugin::mapper::Guest;
use serde_json::{Value, json};

struct QwenMapper;

impl Guest for QwenMapper {
    fn map_request(input: String) -> String {
        let mut req: Value = match serde_json::from_str(&input) {
            Ok(v) => v,
            Err(_) => return input, // 解析失败，原样返回
        };

        let obj = match req.as_object_mut() {
            Some(o) => o,
            None => return input,
        };

        // ── 1. 映射 thinking → enable_thinking ──
        if let Some(thinking) = obj.remove("thinking") {
            let enabled = thinking
                .get("type")
                .and_then(|t| t.as_str())
                .map(|t| t == "enabled")
                .unwrap_or(false);
            obj.insert("enable_thinking".into(), json!(enabled));
        }

        // ── 2. tools 为空时移除，避免千问报错 ──
        let tools_empty = obj
            .get("tools")
            .and_then(|t| t.as_array())
            .map_or(true, |a| a.is_empty());

        if tools_empty {
            obj.remove("tools");
            obj.remove("tool_choice");
        }

        // ── 3. stream_options 自动补全 ──
        let is_stream = obj
            .get("stream")
            .and_then(|s| s.as_bool())
            .unwrap_or(false);

        if is_stream && !obj.contains_key("stream_options") {
            obj.insert("stream_options".into(), json!({"include_usage": true}));
        }

        // ── 4. 清理值为 null 的可选字段 ──
        // 千问对某些 null 字段会报错，统一清理
        let null_keys: Vec<String> = obj
            .iter()
            .filter(|(_, v)| v.is_null())
            .map(|(k, _)| k.clone())
            .collect();

        for key in null_keys {
            obj.remove(&key);
        }

        serde_json::to_string(&req).unwrap_or(input)
    }

    fn map_response(input: String) -> String {
        let mut res: Value = match serde_json::from_str(&input) {
            Ok(v) => v,
            Err(_) => return input,
        };

        if let Some(choices) = res.get_mut("choices").and_then(|c| c.as_array_mut()) {
            for choice in choices.iter_mut() {
                let msg = match choice.get_mut("message").and_then(|m| m.as_object_mut()) {
                    Some(m) => m,
                    None => continue,
                };

                // ── 1. 确保 reasoning_content 存在 ──
                if !msg.contains_key("reasoning_content") {
                    msg.insert("reasoning_content".into(), Value::Null);
                }

                // ── 2. tool_calls 补全 index ──
                if let Some(tool_calls) = msg.get_mut("tool_calls").and_then(|t| t.as_array_mut())
                {
                    for (i, tc) in tool_calls.iter_mut().enumerate() {
                        if let Some(obj) = tc.as_object_mut() {
                            if !obj.contains_key("index") {
                                obj.insert("index".into(), json!(i));
                            }
                        }
                    }
                }
            }
        }

        serde_json::to_string(&res).unwrap_or(input)
    }

    fn map_stream_line(line: String) -> String {
        // ── 提取 JSON 部分 ──
        let trimmed = line.trim();

        // 快速短路：空行、[DONE]、不含 data: 前缀的行
        if trimmed.is_empty()
            || trimmed == "data: [DONE]"
            || trimmed == "[DONE]"
            || !trimmed.starts_with("data:")
        {
            return line;
        }
        
        let (prefix, json_str) = if let Some(rest) = trimmed.strip_prefix("data:") {
            ("data: ", rest.trim())
        } else {
            ("", trimmed)
        };

        // [DONE] 或空行直接透传
        if json_str.is_empty() || json_str == "[DONE]" {
            return line;
        }

        let mut chunk: Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(_) => return line, // 解析失败，原样返回
        };

        // ── 0. usage 字段重命名：千问 DashScope 返回 input_tokens/output_tokens，
        //    但流解码器的 Usage 结构体期望 prompt_tokens/completion_tokens ──
        if let Some(usage) = chunk.get_mut("usage").and_then(|u| u.as_object_mut()) {
            if let Some(v) = usage.remove("input_tokens") {
                usage.insert("prompt_tokens".into(), v);
            }
            if let Some(v) = usage.remove("output_tokens") {
                usage.insert("completion_tokens".into(), v);
            }
        }

        // ── 处理 choices 数组 ──
        let choices = match chunk.get_mut("choices").and_then(|c| c.as_array_mut()) {
            Some(c) => c,
            // usage-only chunk（choices 为空数组），直接透传
            None => return format_sse(prefix, &chunk, &line),
        };

        // choices 为空数组（最后的 usage chunk），直接透传
        if choices.is_empty() {
            return format_sse(prefix, &chunk, &line);
        }

        for choice in choices.iter_mut() {
            let delta = match choice.get_mut("delta").and_then(|d| d.as_object_mut()) {
                Some(d) => d,
                None => continue,
            };

            // ── 1. 确保 reasoning_content 存在 ──
            if !delta.contains_key("reasoning_content") {
                delta.insert("reasoning_content".into(), Value::Null);
            }

            // ── 2-4. 补全 tool_calls 字段 ──
            if let Some(tool_calls) = delta.get_mut("tool_calls").and_then(|t| t.as_array_mut()) {
                for (i, tc) in tool_calls.iter_mut().enumerate() {
                    let tc_obj = match tc.as_object_mut() {
                        Some(o) => o,
                        None => continue,
                    };

                    // 补充 index 字段
                    if !tc_obj.contains_key("index") {
                        tc_obj.insert("index".into(), json!(i));
                    }

                    // function 对象及其子字段
                    if !tc_obj.contains_key("function") {
                        tc_obj.insert(
                            "function".into(),
                            json!({"name": "", "arguments": ""}),
                        );
                    } else if let Some(func) =
                        tc_obj.get_mut("function").and_then(|f| f.as_object_mut())
                    {
                        if !func.contains_key("name") {
                            func.insert("name".into(), json!(""));
                        }
                        if !func.contains_key("arguments") {
                            func.insert("arguments".into(), json!(""));
                        }
                    }
                }
            }
        }

        format_sse(prefix, &chunk, &line)
    }
}

/// 重新序列化为 SSE 行，失败则返回原始行
fn format_sse(prefix: &str, value: &Value, fallback: &str) -> String {
    match serde_json::to_string(value) {
        Ok(s) => format!("{}{}", prefix, s),
        Err(_) => fallback.to_string(),
    }
}

export!(QwenMapper);
