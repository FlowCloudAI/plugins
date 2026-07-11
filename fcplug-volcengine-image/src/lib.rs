wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "api",
});

use crate::exports::mapper::plugin::mapper::Guest;

struct VolcengineImagePlugin;

impl Guest for VolcengineImagePlugin {
    fn map_request(input: String) -> String {
        input
    }

    fn map_response(input: String) -> String {
        input
    }

    fn map_stream_line(line: String) -> String {
        line
    }
}

export!(VolcengineImagePlugin);
