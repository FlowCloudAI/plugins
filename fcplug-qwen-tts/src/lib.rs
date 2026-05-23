wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "api",
});

use crate::exports::mapper::plugin::mapper::Guest;
use serde_json::{Value, json};

struct QwenTTSPlugin;

impl Guest for QwenTTSPlugin {
    /// MiniMax TTSRequest → 千问 TTS 请求
    fn map_request(input: String) -> String {
        let src: Value = match serde_json::from_str(&input) {
            Ok(v) => v,
            Err(_) => return input,
        };

        let model = src["model"].as_str().unwrap_or("qwen3-tts-flash");
        let text = src["text"].as_str().unwrap_or("");
        let voice = src["voice_setting"]["voice_id"]
            .as_str()
            .unwrap_or("Cherry");
        let language_type = src["language_boost"].as_str().unwrap_or("Auto");

        let mut qwen_req = json!({
            "model": model,
            "input": {
                "text": text,
                "voice": voice,
                "language_type": language_type
            }
        });

        if let Some(stream) = src["stream"].as_bool() {
            qwen_req["stream"] = json!(stream);
        }

        serde_json::to_string(&qwen_req).unwrap_or(input)
    }

    /// 千问 TTS 响应 → MiniMax TTSResponse 格式
    fn map_response(input: String) -> String {
        let src: Value = match serde_json::from_str(&input) {
            Ok(v) => v,
            Err(_) => return input,
        };

        let request_id = src["request_id"].as_str().unwrap_or("");

        // 千问成功时无 code 字段；失败时 code 非空
        let err_code = src.get("code").and_then(|v| v.as_str()).unwrap_or("");
        if !err_code.is_empty() {
            let message = src["message"].as_str().unwrap_or("unknown error");
            return serde_json::to_string(&json!({
                "data": null,
                "trace_id": request_id,
                "base_resp": {
                    "status_code": 1000,
                    "status_msg": format!("{}: {}", err_code, message)
                }
            }))
            .unwrap_or(input);
        }

        let audio = &src["output"]["audio"];

        let resp = json!({
            "data": {
                "audio": audio["data"],
                "url": audio["url"],
                "status": 2
            },
            "extra_info": {
                "usage_characters": src["usage"]["characters"],
                "audio_format": "wav"
            },
            "trace_id": audio["id"].as_str().unwrap_or(request_id),
            "base_resp": {
                "status_code": 0,
                "status_msg": "success"
            }
        });

        serde_json::to_string(&resp).unwrap_or(input)
    }

    fn map_stream_line(input: String) -> String {
        let trimmed = input.trim();
        if let Some(json_str) = trimmed.strip_prefix("data:") {
            let json_str = json_str.trim();
            if json_str == "[DONE]" {
                return input;
            }
            let mapped = Self::map_response(json_str.to_string());
            return format!("data: {}", mapped);
        }
        input
    }
}

export!(QwenTTSPlugin);
