wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "api",
});

use crate::exports::mapper::plugin::mapper::Guest;
use serde_json::{Value, json};

struct ZhipuTtsPlugin;

impl Guest for ZhipuTtsPlugin {
    fn map_request(input: String) -> String {
        let src: Value = match serde_json::from_str(&input) {
            Ok(value) => value,
            Err(_) => return input,
        };

        let voice_setting = src.get("voice_setting").unwrap_or(&Value::Null);
        let request = json!({
            "model": src.get("model").and_then(Value::as_str).unwrap_or("glm-tts"),
            "input": src.get("text").and_then(Value::as_str).unwrap_or(""),
            "voice": voice_setting.get("voice_id").and_then(Value::as_str).unwrap_or("tongtong"),
            "speed": voice_setting.get("speed").and_then(Value::as_f64).unwrap_or(1.0),
            "volume": voice_setting.get("vol").and_then(Value::as_f64).unwrap_or(1.0),
            "stream": true,
            "encode_format": "hex",
            "response_format": "pcm",
            "watermark_enabled": false
        });

        serde_json::to_string(&request).unwrap_or(input)
    }

    fn map_response(input: String) -> String {
        if let Ok(error) = serde_json::from_str::<Value>(&input) {
            let message = error
                .pointer("/error/message")
                .or_else(|| error.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("智谱 TTS 请求失败");
            return failure(message, &input);
        }

        let mut audio = String::new();
        let mut trace_id = String::new();
        let mut sample_rate = None;

        for part in input.split("data:").skip(1) {
            let part = part.trim();
            if part.is_empty() || part == "[DONE]" {
                continue;
            }
            let Ok(chunk) = serde_json::from_str::<Value>(part) else {
                continue;
            };
            if trace_id.is_empty() {
                trace_id = chunk
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
            }
            let Some(delta) = chunk.pointer("/choices/0/delta") else {
                continue;
            };
            if let Some(content) = delta.get("content").and_then(Value::as_str) {
                audio.push_str(content);
            }
            sample_rate =
                sample_rate.or_else(|| delta.get("return_sample_rate").and_then(Value::as_u64));
        }

        if audio.is_empty() {
            return failure("智谱 TTS 未返回音频数据", &input);
        }

        serde_json::to_string(&json!({
            "data": {
                "audio": audio,
                "status": 2,
                "url": null
            },
            "extra_info": {
                "audio_format": "pcm",
                "audio_sample_rate": sample_rate
            },
            "trace_id": trace_id,
            "base_resp": {
                "status_code": 0,
                "status_msg": "success"
            }
        }))
        .unwrap_or(input)
    }

    fn map_stream_line(line: String) -> String {
        line
    }
}

fn failure(message: &str, fallback: &str) -> String {
    serde_json::to_string(&json!({
        "data": null,
        "trace_id": null,
        "base_resp": {
            "status_code": 1000,
            "status_msg": message
        }
    }))
    .unwrap_or_else(|_| fallback.to_string())
}

export!(ZhipuTtsPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_request_and_collects_concatenated_sse_audio() {
        let request = ZhipuTtsPlugin::map_request(
            json!({
                "model": "glm-tts",
                "text": "你好",
                "voice_setting": {"voice_id": "tongtong", "speed": 1.2}
            })
            .to_string(),
        );
        let request: Value = serde_json::from_str(&request).unwrap();
        assert_eq!(request["input"], "你好");
        assert_eq!(request["stream"], true);
        assert_eq!(request["encode_format"], "hex");
        assert_eq!(request["watermark_enabled"], false);

        let response = ZhipuTtsPlugin::map_response(
            concat!(
                "data: {\"id\":\"tts-1\",\"choices\":[{\"delta\":{\"content\":\"aabb\",\"return_sample_rate\":24000}}]}",
                "data: {\"id\":\"tts-1\",\"choices\":[{\"delta\":{\"content\":\"ccdd\"}}]}",
                "data: [DONE]"
            )
            .to_string(),
        );
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["data"]["audio"], "aabbccdd");
        assert_eq!(response["extra_info"]["audio_sample_rate"], 24000);
    }
}
