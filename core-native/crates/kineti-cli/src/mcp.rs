//! Native MCP stdio interface; no model turn and no automatic interception.
use crate::{
    gate, host_control, receipt, spend,
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
            "kineti_host_status",
            "Read experimental host capabilities; no task authority or privileged control",
            vec![],
        ),
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
        json!({"name":name,"description":description,"inputSchema":{"type":"object","properties":props,"required":required,"additionalProperties":false},"annotations":{"readOnlyHint":(["kineti_host_status","kineti_status","kineti_spend_status","kineti_evidence_check","kineti_verify_gate_status"].contains(name)),"destructiveHint":*name=="kineti_saga_rollback","openWorldHint":false}})
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
        "kineti_host_status" => Ok(host_control::status()),
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
    serve_bound(root, None)
}
/// Task-only catalog cannot execute arbitrary argv or grant owner authority.
pub fn task_tools() -> Value {
    json!(["status", "run", "check", "recover", "cancel"].iter().map(|m| {
        let mut tool = json!({"name":format!("kineti_task_{m}"),"description":"Use the fixed owner-approved task binding","inputSchema":{"type":"object","properties":{},"additionalProperties":false}});
        if *m == "status" {
            tool["annotations"] = json!({"readOnlyHint":true,"destructiveHint":false,"openWorldHint":false});
        }
        tool
    }).collect::<Vec<_>>())
}
/// Validate the actual app-server-started sidecar, not merely local discovery.
pub fn validate_task_host_catalog(status: &Value) -> Result<()> {
    let valid = (|| {
        let rows = status["data"].as_array()?;
        if rows.len() != 1 || !status["nextCursor"].is_null() {
            return None;
        }
        let row = &rows[0];
        if row["name"] != "kineti_task"
            || row["runtimeStatus"] != "connected"
            || !row["toolsError"].is_null()
        {
            return None;
        }
        let actual = row["tools"].as_object()?;
        let expected = task_tools();
        if actual.len() != expected.as_array()?.len() {
            return None;
        }
        for tool in expected.as_array()? {
            let found = actual.get(tool["name"].as_str()?)?;
            if found["name"] != tool["name"]
                || found["inputSchema"] != tool["inputSchema"]
                || found["annotations"] != tool["annotations"]
            {
                return None;
            }
        }
        Some(())
    })();
    if valid.is_none() {
        return store::fail(3, "deny: app-server task MCP catalog is missing, disconnected or exceeds the fixed binding");
    }
    Ok(())
}

/// Dispatch a scope-free operation through the fixed owner binding.
pub fn task_call(
    root: &Path,
    socket: &Path,
    id: &str,
    session: &str,
    name: &str,
    args: &Value,
) -> Result<Value> {
    if args.as_object().is_none_or(|o| !o.is_empty()) {
        return store::fail(2, "task tools accept no caller scope or command overrides");
    }
    let method = name
        .strip_prefix("kineti_task_")
        .filter(|m| ["status", "run", "check", "recover", "cancel"].contains(m))
        .ok_or_else(|| Error(2, "tool outside task-only catalog".into()))?;
    crate::task::request(socket, id, session, method, root)
}
/// Serve only the fixed task catalog over stdio.
pub fn serve_task(root: &Path, socket: &Path, id: &str, session: &str) -> Result<()> {
    crate::task_grant::id(id)?;
    crate::task_grant::id(session)?;
    serve_bound(root, Some(TaskBinding::Socket(socket, id, session)))
}
/// Serve the same task-only MCP tools through owner-created inherited pipes.
pub fn serve_task_pipe(
    root: &Path,
    read_fd: i32,
    write_fd: i32,
    id: &str,
    session: &str,
) -> Result<()> {
    crate::task_grant::id(id)?;
    crate::task_grant::id(session)?;
    let channel = crate::task_pipe::Channel::inherited(read_fd, write_fd)?;
    serve_bound(root, Some(TaskBinding::Pipe(channel, id, session)))
}
enum TaskBinding<'a> {
    Socket(&'a Path, &'a str, &'a str),
    Pipe(crate::task_pipe::Channel, &'a str, &'a str),
}
fn serve_bound(root: &Path, mut binding: Option<TaskBinding<'_>>) -> Result<()> {
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
            json!({"result":{"protocolVersion":request["params"]["protocolVersion"],"capabilities":{"tools":{}},"serverInfo":{"name":"kineti","version":env!("CARGO_PKG_VERSION")}}})
        } else if !initialized {
            json!({"error":{"code":-32002,"message":"initialize required"}})
        } else {
            match method {
                "tools/list" => {
                    json!({"result":{"tools":if binding.is_some(){task_tools()}else{tools()}}})
                }
                "tools/call" => {
                    let args = request["params"]
                        .get("arguments")
                        .cloned()
                        .unwrap_or(json!({}));
                    let name = request["params"]["name"].as_str().unwrap_or("");
                    match if let Some(binding) = binding.as_mut() {
                        match binding {
                            TaskBinding::Socket(socket, id, session) => {
                                task_call(root, socket, id, session, name, &args)
                            }
                            TaskBinding::Pipe(channel, id, session) => {
                                if args.as_object().is_none_or(|o| !o.is_empty()) {
                                    store::fail(
                                        2,
                                        "task tools accept no caller scope or command overrides",
                                    )
                                } else if let Some(method) = name.strip_prefix("kineti_task_") {
                                    channel.request(id, session, method, root)
                                } else {
                                    store::fail(2, "tool outside task-only catalog")
                                }
                            }
                        }
                    } else {
                        call(root, name, &args)
                    } {
                        Ok(v) => {
                            json!({"result":{"content":[{"type":"text","text":v.to_string()}],"isError":crate::task::blocked(&v)}})
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

#[cfg(test)]
mod task_host_catalog_tests {
    use super::*;
    #[test]
    fn actual_host_catalog_rejects_scope_and_consent_drift() {
        let mut tools = serde_json::Map::new();
        for tool in task_tools().as_array().unwrap() {
            tools.insert(tool["name"].as_str().unwrap().to_string(), tool.clone());
        }
        let baseline = json!({"data":[{"name":"kineti_task","runtimeStatus":"connected","tools":tools,"toolsError":null}],"nextCursor":null});
        assert!(validate_task_host_catalog(&baseline).is_ok());
        for pointer in [
            "/data/0/tools/kineti_task_status/annotations/readOnlyHint",
            "/data/0/tools/kineti_task_run/annotations/readOnlyHint",
        ] {
            let mut changed = baseline.clone();
            changed.pointer_mut("/data/0/tools").unwrap()[if pointer.contains("status") {
                "kineti_task_status"
            } else {
                "kineti_task_run"
            }]["annotations"]["readOnlyHint"] = json!(false);
            assert!(validate_task_host_catalog(&changed).is_err());
        }
        let mut extra = baseline.clone();
        extra["data"][0]["tools"]["kineti_execute"] = json!({});
        assert!(validate_task_host_catalog(&extra).is_err());
        let mut scope = baseline.clone();
        scope["data"][0]["tools"]["kineti_task_run"]["inputSchema"]["additionalProperties"] =
            json!(true);
        assert!(validate_task_host_catalog(&scope).is_err());
        let mut disconnected = baseline.clone();
        disconnected["data"][0]["runtimeStatus"] = json!("failed");
        assert!(validate_task_host_catalog(&disconnected).is_err());
        let mut hidden = baseline.clone();
        hidden["nextCursor"] = json!("more-tools");
        assert!(validate_task_host_catalog(&hidden).is_err());
    }
}
