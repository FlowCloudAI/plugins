wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "api",
});

use crate::exports::mapper::plugin::mapper::Guest;
use serde_json::{Value, json};

struct MiniMaxImageMapper;

impl Guest for MiniMaxImageMapper {
    fn map_request(input: String) -> String {
        let source: Value = match serde_json::from_str(&input) {
            Ok(value) => value,
            Err(_) => return input,
        };

        let model = source
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("image-01");
        let size = source.get("size").and_then(Value::as_str).unwrap_or("1:1");
        let mut request = json!({
            "model": model,
            "prompt": source.get("prompt").and_then(Value::as_str).unwrap_or(""),
            "response_format": map_response_format(source.get("response_format")),
            "n": requested_count(&source),
            "prompt_optimizer": source.get("optimize_prompt_options").is_some_and(|value| !value.is_null()),
            "aigc_watermark": source.get("watermark").and_then(Value::as_bool).unwrap_or(false)
        });

        map_size(&mut request, model, size);
        if let Some(image) = first_image(source.get("image")) {
            request["subject_reference"] = json!([{
                "type": "character",
                "image_file": image
            }]);
        }

        serde_json::to_string(&request).unwrap_or(input)
    }

    fn map_response(input: String) -> String {
        let source: Value = match serde_json::from_str(&input) {
            Ok(value) => value,
            Err(_) => return input,
        };

        let status_code = source
            .pointer("/base_resp/status_code")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        if status_code != 0 {
            return json!({
                "data": null,
                "error": {
                    "code": status_code.to_string(),
                    "message": source.pointer("/base_resp/status_msg").and_then(Value::as_str).unwrap_or("MiniMax 图片生成失败")
                }
            })
            .to_string();
        }

        let mut images = Vec::new();
        if let Some(urls) = source.pointer("/data/image_urls").and_then(Value::as_array) {
            images.extend(
                urls.iter()
                    .filter_map(Value::as_str)
                    .map(|url| json!({"url": url})),
            );
        }
        if let Some(base64_images) = source
            .pointer("/data/image_base64")
            .and_then(Value::as_array)
        {
            images.extend(
                base64_images
                    .iter()
                    .filter_map(Value::as_str)
                    .map(|data| json!({"b64_json": data})),
            );
        }

        json!({
            "data": images,
            "error": null
        })
        .to_string()
    }

    fn map_stream_line(line: String) -> String {
        line
    }
}

fn map_response_format(value: Option<&Value>) -> &'static str {
    match value.and_then(Value::as_str) {
        Some("b64_json" | "base64") => "base64",
        _ => "url",
    }
}

fn requested_count(source: &Value) -> u64 {
    source
        .pointer("/sequential_image_generation_options/max_images")
        .and_then(Value::as_u64)
        .unwrap_or(1)
        .clamp(1, 9)
}

fn first_image(value: Option<&Value>) -> Option<&str> {
    match value? {
        Value::String(image) => Some(image),
        Value::Array(images) => images.first().and_then(Value::as_str),
        _ => None,
    }
}

fn map_size(request: &mut Value, model: &str, size: &str) {
    const RATIOS: [&str; 8] = ["1:1", "16:9", "4:3", "3:2", "2:3", "3:4", "9:16", "21:9"];
    if RATIOS.contains(&size) && (size != "21:9" || model == "image-01") {
        request["aspect_ratio"] = json!(size);
        return;
    }

    if model == "image-01"
        && let Some((width, height)) = parse_dimensions(size)
        && (512..=2048).contains(&width)
        && (512..=2048).contains(&height)
        && width % 8 == 0
        && height % 8 == 0
    {
        request["width"] = json!(width);
        request["height"] = json!(height);
        return;
    }

    request["aspect_ratio"] = json!("1:1");
}

fn parse_dimensions(size: &str) -> Option<(u64, u64)> {
    let (width, height) = size.split_once(['x', '*'])?;
    Some((width.parse().ok()?, height.parse().ok()?))
}

export!(MiniMaxImageMapper);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_seedream_request_to_minimax_image() {
        let mapped = MiniMaxImageMapper::map_request(
            json!({
                "model": "image-01",
                "prompt": "图书馆里的女孩",
                "image": ["https://example.com/person.png", "https://example.com/ignored.png"],
                "size": "1024x1536",
                "response_format": "b64_json",
                "watermark": true,
                "sequential_image_generation_options": {"max_images": 12}
            })
            .to_string(),
        );
        let mapped: Value = serde_json::from_str(&mapped).unwrap();

        assert_eq!(mapped["width"], 1024);
        assert_eq!(mapped["height"], 1536);
        assert_eq!(mapped["response_format"], "base64");
        assert_eq!(mapped["n"], 9);
        assert_eq!(mapped["aigc_watermark"], true);
        assert_eq!(
            mapped["subject_reference"][0]["image_file"],
            "https://example.com/person.png"
        );
    }

    #[test]
    fn normalizes_url_and_base64_responses() {
        let mapped = MiniMaxImageMapper::map_response(
            json!({
                "data": {
                    "image_urls": ["https://example.com/a.jpg"],
                    "image_base64": ["YWJj"]
                },
                "base_resp": {"status_code": 0, "status_msg": "success"}
            })
            .to_string(),
        );
        let mapped: Value = serde_json::from_str(&mapped).unwrap();

        assert_eq!(mapped["data"][0]["url"], "https://example.com/a.jpg");
        assert_eq!(mapped["data"][1]["b64_json"], "YWJj");
        assert!(mapped["error"].is_null());
    }
}
