//! Native MCP stdio interface; no model turn and no automatic interception.
use crate::{
    gate, receipt, spend,
    store::{self, Error, Result},
    undo,
};
use serde_json::{json, Value};
use std::{
    io::{self, BufRead, Write},
    path::Path,
};
fn text<'a>(a: &'a Value, k: &str) -> Result<&'a str> {
    a[k].as_str()
        .ok_or_else(|| Error(2, format!("required string: {k}")))
}
fn strings(a: &Value, k: &str) -> Result<Vec<String>> {
    a[k].as_array()
        .ok_or_else(|| Error(2, format!("required argv array: {k}")))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(String::from)
                .ok_or_else(|| Error(2, "array member must be string".into()))
        })
        .collect()
}
/// Exposed gate tools and native input schemas.
pub fn tools() -> Value {
    let specs = [
        (
            "kineti_status",
            "Read project and reported spending state",
            vec![],
        ),
        (
            "kineti_evidence_record",
            "Execute exact argv and record declared test inputs",
            vec!["label", "command", "required_tests"],
        ),
        (
            "kineti_evidence_check",
            "Check trusted receipt and freshness",
            vec!["label"],
        ),
        (
            "kineti_spend_record",
            "Record an explicit reported USD amount",
            vec!["stage", "usd"],
        ),
        ("kineti_spend_status", "Read reported spending", vec![]),
        (
            "kineti_saga_push",
            "Register an exact argv undo",
            vec!["label", "command"],
        ),
        (
            "kineti_saga_rollback",
            "Preview or execute registered undo",
            vec![],
        ),
        (
            "kineti_verify_gate_status",
            "Check participating CI policy",
            vec!["required_checks"],
        ),
    ];
    json!(specs.iter().map(|(name,description,required)|{
        let mut props=json!({});for k in required {props[*k]=if ["command","required_tests","required_checks"].contains(k){json!({"type":"array","items":{"type":"string"}})}else{json!({"type":"string"})};}
        if *name=="kineti_saga_rollback"{props["confirm"]=json!({"type":"boolean"});props["run_id"]=json!({"type":"string"});}
        json!({"name":name,"description":description,"inputSchema":{"type":"object","properties":props,"required":required,"additionalProperties":false},"annotations":{"readOnlyHint":(["kineti_status","kineti_spend_status","kineti_evidence_check","kineti_verify_gate_status"].contains(name)),"destructiveHint":*name=="kineti_saga_rollback","openWorldHint":false}})
    }).collect::<Vec<_>>())
}
/// Dispatch only declared fields; malformed requests cannot trigger writes.
pub fn call(root: &Path, name: &str, args: &Value) -> Result<Value> {
    let catalog = tools();
    let spec = catalog
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == name)
        .ok_or_else(|| Error(2, format!("removed or unknown MCP tool: {name}")))?;
    let object = args
        .as_object()
        .ok_or_else(|| Error(2, "MCP arguments must be an object".into()))?;
    let props = spec["inputSchema"]["properties"].as_object().unwrap();
    for (k, v) in object {
        let ty = props
            .get(k)
            .and_then(|p| p["type"].as_str())
            .ok_or_else(|| Error(2, format!("unknown MCP argument {k}")))?;
        if match ty {
            "string" => !v.is_string(),
            "boolean" => !v.is_boolean(),
            "array" => v
                .as_array()
                .is_none_or(|a| a.iter().any(|v| !v.is_string())),
            _ => true,
        } {
            return store::fail(2, format!("invalid MCP argument {k}"));
        }
    }
    for k in spec["inputSchema"]["required"].as_array().unwrap() {
        if !object.contains_key(k.as_str().unwrap()) {
            return store::fail(2, format!("missing MCP argument {k}"));
        }
    }
    match name {
        "kineti_status" => {
            Ok(json!({"state":store::state(root)?,"spend":spend::inspect(root,false)?}))
        }
        "kineti_evidence_record" => receipt::run(
            root,
            receipt::Run {
                label: text(args, "label")?.into(),
                argv: strings(args, "command")?,
                tests: strings(args, "required_tests")?,
                timeout: 60,
                kind: "test".into(),
                ..Default::default()
            },
        ),
        "kineti_evidence_check" => receipt::check_kind(root, text(args, "label")?, 240, None, true),
        "kineti_spend_record" => spend::charge(
            root,
            text(args, "stage")?,
            "reported",
            0,
            0,
            text(args, "usd")?,
        ),
        "kineti_spend_status" => spend::inspect(root, false),
        "kineti_saga_push" => undo::push(
            root,
            text(args, "label")?,
            "default",
            strings(args, "command")?,
        ),
        "kineti_saga_rollback" => undo::rollback(
            root,
            args["run_id"].as_str().unwrap_or("default"),
            args["confirm"] == true,
        ),
        "kineti_verify_gate_status" => gate::ci(root, strings(args, "required_checks")?, None, 240),
        _ => unreachable!(),
    }
}
/// Run stdio JSON-RPC. Initialization is required before discovery or calls.
pub fn serve(root: &Path) -> Result<()> {
    let mut initialized = false;
    for line in io::stdin().lock().lines() {
        let line = line?;
        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                println!(
                    "{}",
                    json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"invalid JSON"}})
                );
                continue;
            }
        };
        if request.get("id").is_none() {
            continue;
        }
        let id = request["id"].clone();
        let method = request["method"].as_str().unwrap_or("");
        let response = if method == "initialize" && !initialized {
            initialized = true;
            json!({"result":{"protocolVersion":request["params"]["protocolVersion"],"capabilities":{"tools":{}},"serverInfo":{"name":"kineti","version":"0.6.0"}}})
        } else if !initialized {
            json!({"error":{"code":-32002,"message":"initialize required"}})
        } else {
            match method {
                "tools/list" => json!({"result":{"tools":tools()}}),
                "tools/call" => {
                    let args = request["params"]
                        .get("arguments")
                        .cloned()
                        .unwrap_or(json!({}));
                    match call(
                        root,
                        request["params"]["name"].as_str().unwrap_or(""),
                        &args,
                    ) {
                        Ok(v) => {
                            json!({"result":{"content":[{"type":"text","text":v.to_string()}],"isError":false}})
                        }
                        Err(e) => {
                            json!({"result":{"content":[{"type":"text","text":json!({"code":e.0,"error":e.1}).to_string()}],"isError":true}})
                        }
                    }
                }
                "ping" => json!({"result":{}}),
                _ => json!({"error":{"code":-32601,"message":"removed or unknown method"}}),
            }
        };
        let mut response = response;
        response["jsonrpc"] = json!("2.0");
        response["id"] = id;
        println!("{response}");
        io::stdout().flush()?;
    }
    Ok(())
}
