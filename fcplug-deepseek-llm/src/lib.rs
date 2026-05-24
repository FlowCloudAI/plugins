wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "api",
});

use crate::exports::mapper::plugin::mapper::Guest;
use serde_json::{Value, json};

struct DeepSeekMapper;

impl Guest for DeepSeekMapper {
    fn map_request(input: String) -> String {
        let mut req: Value = match serde_json::from_str(&input) {
            Ok(v) => v,
            Err(_) => return input,
        };

        let obj = match req.as_object_mut() {
            Some(o) => o,
            None => return input,
        };

        // ── 1. tools 为空时移除，避免 API 报错 ──
        let tools_empty = obj
            .get("tools")
            .and_then(|t| t.as_array())
            .map_or(true, |a| a.is_empty());

        if tools_empty {
            obj.remove("tools");
            obj.remove("tool_choice");
        }

        // ── 2. 映射 thinking_effort → DeepSeek reasoning_effort ──
        // DeepSeek V4 官方只提供 high / max；为兼容现有枚举，low 和 medium 按 high 处理，xhigh 按 max 处理。
        if let Some(effort) = obj
            .remove("thinking_effort")
            .and_then(|value| value.as_str().map(str::to_string))
        {
            let reasoning_effort = match effort.as_str() {
                "low" | "medium" | "high" => Some("high"),
                "xhigh" => Some("max"),
                _ => None,
            };
            if let Some(value) = reasoning_effort {
                obj.insert("reasoning_effort".into(), json!(value));
            }
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
                if let Some(tool_calls) = msg.get_mut("tool_calls").and_then(|t| t.as_array_mut()) {
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

        if json_str.is_empty() || json_str == "[DONE]" {
            return line;
        }

        let mut chunk: Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(_) => return line,
        };

        let has_usage = chunk.get("usage").is_some();

        let choices = match chunk.get_mut("choices").and_then(|c| c.as_array_mut()) {
            Some(c) => c,
            None => return format_sse(prefix, &chunk, &line),
        };

        if choices.is_empty() {
            // ── DeepSeek 某些代理会在末尾只返回 usage chunk，缺少显式 finish_reason / [DONE] ──
            // 为避免上层一直卡在流式状态，这里把该尾块补成一个 stop 终止块。
            if has_usage {
                *choices = vec![json!({
                    "index": 0,
                    "delta": {
                        "content": "",
                        "reasoning_content": Value::Null,
                    },
                    "finish_reason": "stop",
                })];
            }
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

            // ── 2. 补全 tool_calls 字段 ──
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

export!(DeepSeekMapper);
