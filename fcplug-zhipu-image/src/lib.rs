wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "api",
});

use crate::exports::mapper::plugin::mapper::Guest;
use serde_json::{Value, json};

struct ZhipuImagePlugin;

impl Guest for ZhipuImagePlugin {
    fn map_request(input: String) -> String {
        let src: Value = match serde_json::from_str(&input) {
            Ok(value) => value,
            Err(_) => return input,
        };

        let model = src
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("glm-image");
        let size = map_size(
            model,
            src.get("size").and_then(Value::as_str).unwrap_or("2K"),
        );

        let request = json!({
            "model": model,
            "prompt": src.get("prompt").and_then(Value::as_str).unwrap_or(""),
            "quality": if model == "glm-image" { "hd" } else { "standard" },
            "size": size,
            "watermark_enabled": src.get("watermark").and_then(Value::as_bool).unwrap_or(true)
        });

        serde_json::to_string(&request).unwrap_or(input)
    }

    fn map_response(input: String) -> String {
        input
    }

    fn map_stream_line(line: String) -> String {
        line
    }
}

fn map_size(model: &str, size: &str) -> String {
    match size {
        "1K" => "1024x1024".to_string(),
        "2K" | "3K" | "4K" if model == "glm-image" => "2048x2048".to_string(),
        "2K" | "3K" | "4K" => "1024x1024".to_string(),
        custom => custom.replace('*', "x"),
    }
}

export!(ZhipuImagePlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_seedream_request_to_glm_image() {
        let mapped = ZhipuImagePlugin::map_request(
            json!({
                "model": "glm-image",
                "prompt": "一只猫",
                "size": "2K",
                "watermark": false
            })
            .to_string(),
        );
        let mapped: Value = serde_json::from_str(&mapped).unwrap();

        assert_eq!(mapped["quality"], "hd");
        assert_eq!(mapped["size"], "2048x2048");
        assert_eq!(mapped["watermark_enabled"], false);
        assert!(mapped.get("image").is_none());
    }
}
