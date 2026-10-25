//! Disposable protocol probe, not the gate implementation or release package.
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.expect("stdin");
        let request: Value = serde_json::from_str(&line).expect("JSON-RPC input");
        let Some(id) = request.get("id") else {
            continue;
        };
        let result = match request["method"].as_str() {
            Some("initialize") => json!({
                "protocolVersion": request["params"]["protocolVersion"],
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "kineti-native-feasibility", "version": "0.0.0"}
            }),
            Some("tools/list") => json!({"tools": [{
                "name": "kineti_feasibility_status",
                "description": "Read-only native protocol probe; not a release gate",
                "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false},
                "annotations": {"readOnlyHint": true, "destructiveHint": false, "openWorldHint": false}
            }]}),
            Some("tools/call") if request["params"]["name"] == "kineti_feasibility_status" => {
                json!({"content": [{"type": "text", "text": "kineti-native-feasibility-ok"}], "isError": false})
            }
            Some("ping") => json!({}),
            _ => {
                println!(
                    "{}",
                    json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": "unsupported feasibility method"}})
                );
                io::stdout().flush().expect("flush");
                continue;
            }
        };
        println!("{}", json!({"jsonrpc": "2.0", "id": id, "result": result}));
        io::stdout().flush().expect("flush");
    }
}
