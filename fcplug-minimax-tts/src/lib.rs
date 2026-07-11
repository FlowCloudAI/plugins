wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "api",
});

use crate::exports::mapper::plugin::mapper::Guest;

struct MiniMaxTtsMapper;

impl Guest for MiniMaxTtsMapper {
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

export!(MiniMaxTtsMapper);
