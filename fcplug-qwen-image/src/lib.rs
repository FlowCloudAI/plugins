wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "api",
});

use crate::exports::mapper::plugin::mapper::Guest;
use serde_json::{Value, json};

struct QwenImagePlugin;

impl Guest for QwenImagePlugin {
    /// 火山方舟 Seedream 请求 → 千问 Qwen-Image 请求
    ///
    /// Seedream 格式:
    /// {
    ///   "model": "doubao-seedream-5-0-260128",
    ///   "prompt": "...",
    ///   "image": "url" | ["url1", "url2"],
    ///   "size": "2K" | "2048x2048",
    ///   "output_format": "png",
    ///   "response_format": "url",
    ///   "watermark": false,
    ///   "sequential_image_generation": "auto",
    ///   "sequential_image_generation_options": { "max_images": 4 },
    ///   "optimize_prompt_options": { "mode": "standard" }
    /// }
    ///
    /// 千问格式:
    /// {
    ///   "model": "qwen-image-2.0-pro",
    ///   "input": {
    ///     "messages": [{
    ///       "role": "user",
    ///       "content": [{ "text": "..." }]
    ///     }]
    ///   },
    ///   "parameters": {
    ///     "size": "2048*2048",
    ///     "n": 1,
    ///     "watermark": false,
    ///     "prompt_extend": true,
    ///     "negative_prompt": "..."
    ///
    /// }
    fn map_request(input: String) -> String {
        let src: Value = match serde_json::from_str(&input) {
            Ok(v) => v,
            Err(_) => return input,
        };

        let model = src["model"].as_str().unwrap_or("qwen-image-2.0-pro");
        let prompt = src
            .get("prompt")
            .and_then(|v| v.as_str())
            .or_else(|| src.get("input").and_then(|v| v.get("prompt")).and_then(|v| v.as_str()))
            .unwrap_or("");

        // size 映射："2K" → "2048*2048"，"3K" → "3072*3072"，"2048x2048" → "2048*2048"
        let size_raw = src["size"].as_str().unwrap_or("2K");
        let size = match size_raw {
            "1K" => "1024*1024".to_string(),
            "2K" => "2048*2048".to_string(),
            "3K" => "3072*3072".to_string(),
            "4K" => "4096*4096".to_string(),
            other => other.replace('x', "*"),
        };

        let watermark = src["watermark"].as_bool().unwrap_or(false);

        // 映射 optimize_prompt_options.mode → prompt_extend
        let prompt_extend = match src["optimize_prompt_options"]["mode"].as_str() {
            Some("fast") => false,
            _ => true,
        };

        // 兼容宿主可能直接透传的 n，同时支持 Seedream 的 sequential_image_generation_options.max_images。
        let mut image_count = src
            .get("n")
            .and_then(|v| v.as_u64())
            .or_else(|| {
                src.get("sequential_image_generation_options")
                    .and_then(|v| v.get("max_images"))
                    .and_then(|v| v.as_u64())
            })
            .unwrap_or(1);

        if matches!(model, "qwen-image-max" | "qwen-image-plus") {
            image_count = 1;
        } else {
            image_count = image_count.clamp(1, 6);
        }

        // 文生图接口要求 content 中有且只有一个 text，图生图则是在同一个数组里追加 image。
        let mut content = vec![json!({ "text": prompt })];

        match &src["image"] {
            Value::String(url) if !url.is_empty() => {
                content.push(json!({ "image": url }));
            }
            Value::Array(urls) if !urls.is_empty() => {
                for url in urls {
                    if let Some(u) = url.as_str() {
                        content.push(json!({ "image": u }));
                    }
                }
            }
            _ => {}
        }

        let mut parameters = json!({
            "size": size,
            "n": image_count,
            "watermark": watermark,
            "prompt_extend": prompt_extend
        });

        // negative_prompt 千问支持但 Seedream 基准格式没有，通过 ext 透传
        if let Some(neg) = src.get("negative_prompt").and_then(|v| v.as_str()) {
            parameters["negative_prompt"] = json!(neg);
        }

        // seed 透传
        if let Some(seed) = src.get("seed").and_then(|v| v.as_u64()) {
            parameters["seed"] = json!(seed);
        }

        let qwen_req = json!({
            "model": model,
            "input": {
                "messages": [{
                    "role": "user",
                    "content": content
                }]
            },
            "parameters": parameters
        });

        serde_json::to_string(&qwen_req).unwrap_or(input)
    }

    /// 千问 Qwen-Image 响应 → 火山方舟 Seedream 响应
    ///
    /// 千问格式:
    /// {
    ///   "output": {
    ///     "choices": [{
    ///       "finish_reason": "stop",
    ///       "message": {
    ///         "role": "assistant",
    ///         "content": [{ "image": "https://..." }]
    ///       }
    ///     }]
    ///   },
    ///   "usage": { "image_count": 1, "width": 2048, "height": 2048 },
    ///   "request_id": "xxx"
    /// }
    ///
    /// Seedream 格式:
    /// {
    ///   "created": ...,
    ///   "data": [{ "url": "...", "size": "2048x2048" }],
    ///   "usage": { "total_tokens": 0 },
    ///   "error": null
    /// }
    fn map_response(input: String) -> String {
        let src: Value = match serde_json::from_str(&input) {
            Ok(v) => v,
            Err(_) => return input,
        };

        // 错误处理: 千问失败时有 code 字段
        let err_code = src.get("code").and_then(|v| v.as_str()).unwrap_or("");
        if !err_code.is_empty() {
            let message = src["message"].as_str().unwrap_or("unknown error");
            let resp = json!({
                "created": null,
                "data": null,
                "error": {
                    "code": err_code,
                    "message": message
                },
                "usage": null
            });
            return serde_json::to_string(&resp).unwrap_or(input);
        }

        // 提取图片 URL 列表
        let mut images: Vec<Value> = Vec::new();

        let usage = &src["usage"];
        let width = usage["width"].as_u64().unwrap_or(0);
        let height = usage["height"].as_u64().unwrap_or(0);
        let size_str = if width > 0 && height > 0 {
            format!("{}x{}", width, height)
        } else {
            String::new()
        };

        // 千问同步接口: output.choices[].message.content[].image
        if let Some(choices) = src["output"]["choices"].as_array() {
            for choice in choices {
                if let Some(content) = choice["message"]["content"].as_array() {
                    for item in content {
                        if let Some(url) = item["image"].as_str() {
                            let mut img = json!({ "url": url });
                            if !size_str.is_empty() {
                                img["size"] = json!(size_str);
                            }
                            // 提取 revised_prompt (千问的 actual_prompt)
                            if let Some(actual) =
                                choice.get("actual_prompt").and_then(|v| v.as_str())
                            {
                                img["revised_prompt"] = json!(actual);
                            }
                            images.push(img);
                        }
                    }
                }
            }
        }

        // 千问异步接口: output.results[].url
        if images.is_empty() {
            if let Some(results) = src["output"]["results"].as_array() {
                for result in results {
                    if let Some(url) = result["url"].as_str() {
                        let mut img = json!({ "url": url });
                        if !size_str.is_empty() {
                            img["size"] = json!(size_str);
                        }
                        if let Some(actual) = result.get("actual_prompt").and_then(|v| v.as_str()) {
                            img["revised_prompt"] = json!(actual);
                        }
                        images.push(img);
                    }
                }
            }
        }

        let image_count = usage["image_count"].as_u64().unwrap_or(images.len() as u64);

        let resp = json!({
            "created": null,
            "data": images,
            "error": null,
            "usage": {
                "prompt_tokens": null,
                "completion_tokens": null,
                "total_tokens": null,
                "image_count": image_count
            }
        });

        serde_json::to_string(&resp).unwrap_or(input)
    }

    /// 流式行映射
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

export!(QwenImagePlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_request_keeps_messages_and_parameters() {
        let input = json!({
            "model": "qwen-image-2.0",
            "prompt": "一只橘猫坐在窗边",
            "size": "2048x2048",
            "watermark": false,
            "negative_prompt": "模糊",
            "seed": 42,
            "sequential_image_generation_options": {
                "max_images": 4
            }
        });

        let mapped = QwenImagePlugin::map_request(input.to_string());
        let mapped: Value = serde_json::from_str(&mapped).unwrap();

        assert_eq!(mapped["input"]["messages"][0]["role"], "user");
        assert_eq!(mapped["input"]["messages"][0]["content"][0]["text"], "一只橘猫坐在窗边");
        assert_eq!(mapped["parameters"]["size"], "2048*2048");
        assert_eq!(mapped["parameters"]["n"], 4);
        assert_eq!(mapped["parameters"]["negative_prompt"], "模糊");
        assert_eq!(mapped["parameters"]["seed"], 42);
    }

    #[test]
    fn map_request_keeps_image_inputs() {
        let input = json!({
            "model": "qwen-image-2.0-pro",
            "prompt": "让两张图融合",
            "image": [
                "https://example.com/1.png",
                "https://example.com/2.png"
            ]
        });

        let mapped = QwenImagePlugin::map_request(input.to_string());
        let mapped: Value = serde_json::from_str(&mapped).unwrap();
        let content = mapped["input"]["messages"][0]["content"].as_array().unwrap();

        assert_eq!(content.len(), 3);
        assert_eq!(content[0]["text"], "让两张图融合");
        assert_eq!(content[1]["image"], "https://example.com/1.png");
        assert_eq!(content[2]["image"], "https://example.com/2.png");
    }
}
