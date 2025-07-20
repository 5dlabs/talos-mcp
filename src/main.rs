use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::process::{Command, Stdio};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::runtime::Runtime;

mod tools;

// Custom error type for production-ready error handling.
#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

// JSON-RPC Success Response structure.
#[derive(Serialize)]
struct RpcSuccessResponse {
    jsonrpc: String,
    result: Value,
    id: Option<Value>,
}

// JSON-RPC Error Response structure.
#[derive(Serialize)]
struct RpcErrorResponse {
    jsonrpc: String,
    error: RpcError,
    id: Option<Value>,
}

// JSON-RPC Request structure.
#[derive(Deserialize)]
struct RpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

// Helper to run talosctl command and capture output.
fn run_talosctl(args: &[&str]) -> Result<String> {
    let talosconfig = env::var("TALOSCONFIG").context("TALOSCONFIG env var not set")?;
    let mut cmd = Command::new("talosctl");
    cmd.arg("--talosconfig").arg(&talosconfig);
    cmd.args(args);
    cmd.stderr(Stdio::piped());
    let output = cmd.output().context("Failed to execute talosctl")?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(anyhow!("talosctl failed: {}", err));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// Helper to run talosctl command and capture stderr output (for health checks).
fn run_talosctl_with_stderr(args: &[&str]) -> Result<String> {
    let talosconfig = env::var("TALOSCONFIG").context("TALOSCONFIG env var not set")?;
    let mut cmd = Command::new("talosctl");
    cmd.arg("--talosconfig").arg(&talosconfig);
    cmd.args(args);
    cmd.stderr(Stdio::piped());
    let output = cmd.output().context("Failed to execute talosctl")?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(anyhow!("talosctl failed: {}", err));
    }
    // For health checks, the useful output is in stderr, not stdout
    Ok(String::from_utf8_lossy(&output.stderr).to_string())
}

// Capabilities advertised by the server with full MCP tool schemas.
fn get_capabilities() -> Value {
    tools::get_all_tool_schemas()
}

// Extract parameters from JSON value into HashMap
fn extract_params(params: Option<&Value>) -> HashMap<String, Value> {
    params
        .and_then(|p| p.as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect()))
        .unwrap_or_default()
}

// Handle system inspection and monitoring methods
fn handle_system_inspection_methods(method: &str, params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "containers" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let kubernetes = params_map.get("kubernetes").and_then(|v| v.as_bool()).unwrap_or(false);
            match node {
                Ok(node) => {
                    let mut args = vec!["--nodes", node, "containers"];
                    if kubernetes {
                        args.push("--kubernetes");
                    }
                    let output = run_talosctl(&args);
                    Some(output.map(|out| json!({"containers": out, "namespace": if kubernetes { "k8s.io" } else { "system" }})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "stats" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let kubernetes = params_map.get("kubernetes").and_then(|v| v.as_bool()).unwrap_or(false);
            match node {
                Ok(node) => {
                    let mut args = vec!["--nodes", node, "stats"];
                    if kubernetes {
                        args.push("--kubernetes");
                    }
                    let output = run_talosctl(&args);
                    Some(output.map(|out| json!({"stats": out, "namespace": if kubernetes { "k8s.io" } else { "system" }})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "memory_verbose" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "memory", "--verbose"]);
                    Some(output.map(|out| json!({"memory_verbose": out})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "get_cpu_memory_usage" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let mem = run_talosctl(&["--nodes", node, "memory"]);
                    let cgroups = run_talosctl(&["--nodes", node, "cgroups", "--preset", "cpu"]);
                    match (mem, cgroups) {
                        (Ok(mem), Ok(cgroups)) => Some(Ok(json!({"memory": mem, "cpu": cgroups}))),
                        (Err(e), _) | (_, Err(e)) => Some(Err(e))
                    }
                }
                Err(e) => Some(Err(e))
            }
        }
        "get_processes" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let sort = params_map.get("sort").and_then(|v| v.as_str()).unwrap_or("rss");
            match node {
                Ok(node) => {
                    let args = vec!["--nodes", node, "processes", "--sort", sort];
                    let output = run_talosctl(&args);
                    Some(output.map(|out| json!({"processes": out, "sort_by": sort})))
                }
                Err(e) => Some(Err(e))
            }
        }
        _ => None
    }
}

// Handle file system operations
fn handle_file_operations_methods(method: &str, params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
                "list" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let path = params_map.get("path").and_then(|v| v.as_str()).unwrap_or("/");
            let long = params_map.get("long").and_then(|v| v.as_bool()).unwrap_or(false);
            let humanize = params_map.get("humanize").and_then(|v| v.as_bool()).unwrap_or(false);
            let recurse = params_map.get("recurse").and_then(|v| v.as_bool()).unwrap_or(false);
            let depth = params_map.get("depth").and_then(|v| v.as_i64()).unwrap_or(1);
            let file_types = params_map.get("type").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>());

                        match node {
                Ok(node) => {
                    let mut args = vec!["--nodes", node, "list", path];
                    let depth_str = depth.to_string();

                    if long {
                        args.push("--long");
                    }

                    if humanize {
                        args.push("--humanize");
                    }

                    // --recurse and --depth are mutually exclusive
                    if recurse {
                        args.push("--recurse");
                    } else if depth != 1 {
                        args.extend(&["--depth", &depth_str]);
                    }

                    if let Some(types) = &file_types {
                        for file_type in types {
                            args.extend(&["--type", file_type]);
                        }
                    }

                    let output = run_talosctl(&args);
                    Some(output.map(|out| json!({
                        "list": out,
                        "path": path,
                        "long": long,
                        "humanize": humanize,
                        "recurse": recurse,
                        "depth": depth,
                        "types": file_types
                    })))
                }
                Err(e) => Some(Err(e))
            }
        }
        "read" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let path = params_map.get("path").and_then(|v| v.as_str()).ok_or(anyhow!("Missing path param"));
            match (node, path) {
                (Ok(node), Ok(path)) => {
                    let output = run_talosctl(&["--nodes", node, "read", path]);
                    Some(output.map(|out| json!({"content": out})))
                }
                (Err(e), _) | (_, Err(e)) => Some(Err(e))
            }
        }
        "copy" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let source = params_map.get("source").and_then(|v| v.as_str()).ok_or(anyhow!("Missing source param"));
            let destination = params_map.get("destination").and_then(|v| v.as_str()).ok_or(anyhow!("Missing destination param"));
            match (node, source, destination) {
                (Ok(node), Ok(source), Ok(destination)) => {
                    let output = run_talosctl(&["--nodes", node, "copy", source, destination]);
                    Some(output.map(|out| json!({"copy": out})))
                }
                (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => Some(Err(e))
            }
        }
        "get_usage" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let path = params_map.get("path").and_then(|v| v.as_str()).unwrap_or("/");
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "usage", path]);
                    Some(output.map(|out| json!({"usage": out})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "get_mounts" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "mounts"]);
                    Some(output.map(|out| json!({"mounts": out})))
                }
                Err(e) => Some(Err(e))
            }
        }
        _ => None
    }
}

// Handle network operations
fn handle_network_operations_methods(method: &str, params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "interfaces" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let namespace = params_map.get("namespace").and_then(|v| v.as_str());
            let output_format = params_map.get("output").and_then(|v| v.as_str()).unwrap_or("table");

            match node {
                Ok(node) => {
                    let mut args = vec!["--nodes", node, "get", "addresses"];

                    if let Some(ns) = namespace {
                        args.extend(&["--namespace", ns]);
                    }

                    args.extend(&["--output", output_format]);

                    let output = run_talosctl(&args);
                    Some(output.map(|out| json!({
                        "interfaces": out,
                        "namespace": namespace,
                        "output_format": output_format
                    })))
                }
                Err(e) => Some(Err(e))
            }
        }
        "routes" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let namespace = params_map.get("namespace").and_then(|v| v.as_str());
            let output_format = params_map.get("output").and_then(|v| v.as_str()).unwrap_or("table");

            match node {
                Ok(node) => {
                    let mut args = vec!["--nodes", node, "get", "routes"];

                    if let Some(ns) = namespace {
                        args.extend(&["--namespace", ns]);
                    }

                    args.extend(&["--output", output_format]);

                    let output = run_talosctl(&args);
                    Some(output.map(|out| json!({
                        "routes": out,
                        "namespace": namespace,
                        "output_format": output_format
                    })))
                }
                Err(e) => Some(Err(e))
            }
        }
        "get_netstat" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "netstat"]);
                    Some(output.map(|out| json!({"netstat": out})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "capture_packets" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let interface = params_map.get("interface").and_then(|v| v.as_str()).unwrap_or("eth0");
            let duration = params_map.get("duration").and_then(|v| v.as_str()).unwrap_or("10s");
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "pcap", "--interface", interface, "--duration", duration]);
                    Some(output.map(|out| json!({"packets": out})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "get_network_io_cgroups" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "cgroups", "--preset", "io"]);
                    Some(output.map(|out| json!({"network_io": out})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "list_network_interfaces" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "list", "/sys/class/net"]);
                    Some(output.map(|out| json!({"interfaces": out})))
                }
                Err(e) => Some(Err(e))
            }
        }
        _ => None
    }
}

// Handle service and logging operations
fn handle_service_log_methods(method: &str, params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "dmesg" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let args = vec!["--nodes", node, "dmesg"];
                    let output = run_talosctl(&args);
                    Some(output.map(|out| json!({
                        "dmesg": out
                    })))
                }
                Err(e) => Some(Err(e))
            }
        }
        "service" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let service = params_map.get("service").and_then(|v| v.as_str()).ok_or(anyhow!("Missing service param"));
            let action = params_map.get("action").and_then(|v| v.as_str()).unwrap_or("status");
            match (node, service) {
                (Ok(node), Ok(service)) => {
                    let output = run_talosctl(&["--nodes", node, "service", service, action]);
                    Some(output.map(|out| json!({"service": out})))
                }
                (Err(e), _) | (_, Err(e)) => Some(Err(e))
            }
        }
        "restart" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let service = params_map.get("service").and_then(|v| v.as_str()).ok_or(anyhow!("Missing service param"));
            match (node, service) {
                (Ok(node), Ok(service)) => {
                    let output = run_talosctl(&["--nodes", node, "service", service, "restart"]);
                    Some(output.map(|out| json!({"restart": out})))
                }
                (Err(e), _) | (_, Err(e)) => Some(Err(e))
            }
        }
        "get_events" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "events"]);
                    Some(output.map(|out| json!({"events": out})))
                }
                Err(e) => Some(Err(e))
            }
        }
        _ => None
    }
}

// Handle storage and hardware methods
fn handle_storage_hardware_methods(method: &str, params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "disks" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let namespace = params_map.get("namespace").and_then(|v| v.as_str());
            let output_format = params_map.get("output").and_then(|v| v.as_str()).unwrap_or("table");

            match node {
                Ok(node) => {
                    let mut args = vec!["--nodes", node, "get", "disks"];

                    if let Some(ns) = namespace {
                        args.extend(&["--namespace", ns]);
                    }

                    args.extend(&["--output", output_format]);

                    let output = run_talosctl(&args);
                    Some(output.map(|out| json!({
                        "disks": out,
                        "namespace": namespace,
                        "output_format": output_format
                    })))
                }
                Err(e) => Some(Err(e))
            }
        }
        "list_disks" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "list", "/sys/block"]);
                    Some(output.map(|out| json!({"disks": out})))
                }
                Err(e) => Some(Err(e))
            }
        }
        _ => None
    }
}

// Handle MCP protocol methods
fn handle_mcp_protocol_methods(method: &str, params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "initialize" => {
            // MCP initialization - validate required fields and return proper server capabilities
            let _protocol_version = params_map.get("protocolVersion")
                .and_then(|v| v.as_str())
                .unwrap_or("2025-06-18");

            // Validate that required fields are present (as per MCP schema)
            if params_map.get("capabilities").is_none() ||
               params_map.get("clientInfo").is_none() ||
               params_map.get("protocolVersion").is_none() {
                return Some(Err(anyhow!("Missing required initialize parameters: capabilities, clientInfo, and protocolVersion are required")));
            }

            Some(Ok(json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {
                    "tools": {
                        "listChanged": true
                    }
                },
                "serverInfo": {
                    "name": "talos-mcp-server",
                    "title": "Talos OS MCP Server",
                    "version": "1.0.0"
                }
            })))
        }
        "notifications/initialized" => {
            // MCP initialized notification - no response should be sent
            None
        }
        method if method.starts_with("notifications/") => {
            // Debug: catch any notifications we might be missing
            None
        }
        "ping" => {
            // MCP ping request - respond with empty object for connection health
            Some(Ok(json!({})))
        }
        "tools/list" => {
            // Return list of available tools with schemas
            Some(Ok(get_capabilities()))
        }
        _ => None
    }
}

// Handle tool invocation
fn handle_tool_invocation(params_map: &HashMap<String, Value>) -> Result<Value> {
    let name = params_map.get("name").and_then(|v| v.as_str()).ok_or(anyhow!("Missing tool name"))?;
    let default_args = json!({});
    let arguments = params_map.get("arguments").unwrap_or(&default_args);

    // Extract arguments as a map for the tool handlers
    let args_map = extract_params(Some(arguments));

    // Try each handler category to find the tool
    let tool_result = if let Some(result) = handle_system_inspection_methods(name, &args_map) {
        Some(result)
    } else if let Some(result) = handle_file_operations_methods(name, &args_map) {
        Some(result)
    } else if let Some(result) = handle_network_operations_methods(name, &args_map) {
        Some(result)
    } else if let Some(result) = handle_service_log_methods(name, &args_map) {
        Some(result)
    } else if let Some(result) = handle_storage_hardware_methods(name, &args_map) {
        Some(result)
    } else {
        let result = handle_core_cluster_methods(name, &args_map);
        if result.is_some() {
            result // Core methods can return None
        } else if let Some(result) = handle_node_management_methods(name, &args_map) {
            Some(result)
        } else if let Some(result) = handle_config_etcd_methods(name, &args_map) {
            Some(result)
        } else {
            Some(Err(anyhow!("Unknown tool: {}", name)))
        }
    };

    match tool_result {
        Some(Ok(content)) => Ok(json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&content).unwrap_or_else(|_| content.to_string())
                }
            ]
        })),
        Some(Err(e)) => Err(e),
        None => Err(anyhow!("Tool {} returned no response", name))
    }
}

// Handle core cluster monitoring methods
fn handle_core_cluster_methods(method: &str, params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "tools/call" => Some(handle_tool_invocation(params_map)),
        "get_version" => {
            let short = params_map.get("short").and_then(|v| v.as_bool()).unwrap_or(false);

            let mut args = vec!["version", "--client"];
            if short {
                args.push("--short");
            }

            let output = run_talosctl(&args);
            Some(output.map(|out| json!({
                "version": out,
                "short_format": short
            })))
        }
                "get_time" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).filter(|s| !s.is_empty());
            let check = params_map.get("check").and_then(|v| v.as_str());

            // Time command requires a node to be specified
            let target_node = match node {
                Some(n) => n,
                None => {
                    return Some(Err(anyhow!("Time command requires a node to be specified. Please provide a node parameter.")));
                }
            };

            let mut args = vec!["--nodes", target_node, "time"];

            if let Some(ntp_server) = check {
                args.extend(&["--check", ntp_server]);
            }

            let output = run_talosctl(&args);
            Some(output.map(|out| json!({
                "time": out,
                "node": target_node,
                "ntp_check": check
            })))
        }
        "get_health" => {
            let control_planes = params_map.get("control_planes")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or(vec!["192.168.1.77"]);

            let worker_nodes = params_map.get("worker_nodes")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>());

            let init_node = params_map.get("init_node").and_then(|v| v.as_str());
            let timeout = params_map.get("timeout").and_then(|v| v.as_str()).unwrap_or("120s");
            let run_e2e = params_map.get("run_e2e").and_then(|v| v.as_bool()).unwrap_or(false);
            let k8s_endpoint = params_map.get("k8s_endpoint").and_then(|v| v.as_str());
            let server = params_map.get("server").and_then(|v| v.as_bool()).unwrap_or(true);

            if control_planes.is_empty() {
                return Some(Err(anyhow!("At least one control plane node must be specified")));
            }

            // Prepare string values that need to live for the entire function
            let control_planes_str = control_planes.join(",");
            let workers_str = worker_nodes.as_ref().map(|w| w.join(","));

            // Build command arguments dynamically
            let mut args = Vec::new();

            // Always specify the first control plane node for --nodes
            args.extend(&["--nodes", control_planes[0]]);

            // Add the health command
            args.push("health");

            // Add control plane nodes
            args.extend(&["--control-plane-nodes", &control_planes_str]);

            // Add worker nodes if specified
            if let Some(ref workers_string) = workers_str {
                args.extend(&["--worker-nodes", workers_string]);
            }

            // Add init node if specified
            if let Some(init) = init_node {
                args.extend(&["--init-node", init]);
            }

            // Add timeout
            args.extend(&["--wait-timeout", timeout]);

            // Add run-e2e flag if true
            if run_e2e {
                args.push("--run-e2e");
            }

            // Add k8s endpoint if specified
            if let Some(endpoint) = k8s_endpoint {
                args.extend(&["--k8s-endpoint", endpoint]);
            }

            // Add server flag (note: --server is default true, --no-server to disable)
            if !server {
                args.push("--server=false");
            }

            let output = run_talosctl_with_stderr(&args);
            match output {
                Ok(out) => Some(Ok(json!({
                    "health": out,
                    "cluster_info": {
                        "control_planes": control_planes,
                        "worker_nodes": worker_nodes,
                        "init_node": init_node,
                        "timeout": timeout,
                        "run_e2e": run_e2e,
                        "k8s_endpoint": k8s_endpoint,
                        "server_side": server
                    }
                }))),
                Err(e) => Some(Err(anyhow!("Health check failed: {}", e))),
            }
        }
        "get_logs" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let service = params_map.get("service").and_then(|v| v.as_str()).ok_or(anyhow!("Missing service param"));
            let tail = params_map.get("tail").and_then(|v| v.as_i64());
            let kubernetes = params_map.get("kubernetes").and_then(|v| v.as_bool()).unwrap_or(false);
            match (node, service) {
                (Ok(node), Ok(service)) => {
                    let mut args = vec!["--nodes", node, "logs", service];

                    let tail_str = tail.map(|t| t.to_string());
                    if let Some(ref tail_count) = tail_str {
                        args.extend(&["--tail", tail_count]);
                    }

                    if kubernetes {
                        args.push("--kubernetes");
                    }

                    let output = run_talosctl(&args);
                    Some(output.map(|out| json!({
                        "logs": out,
                        "service": service,
                        "tail_lines": tail,
                        "namespace": if kubernetes { "k8s.io" } else { "system" }
                    })))
                }
                (Err(e), _) | (_, Err(e)) => Some(Err(e))
            }
        }
        _ => None
    }
}

// Handle node management methods
fn handle_node_management_methods(method: &str, params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "reboot_node" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "reboot"]);
                    Some(output.map(|_| json!({"status": "reboot initiated"})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "shutdown_node" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "shutdown"]);
                    Some(output.map(|_| json!({"status": "node shutdown initiated"})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "reset_node" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "reset"]);
                    Some(output.map(|_| json!({"status": "node reset initiated"})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "upgrade_node" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let image = params_map.get("image").and_then(|v| v.as_str()).unwrap_or("ghcr.io/siderolabs/installer:latest");
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "upgrade", "--image", image]);
                    Some(output.map(|_| json!({"status": "upgrade initiated"})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "upgrade_k8s" => {
            let from = params_map.get("from").and_then(|v| v.as_str()).unwrap_or("1.28.0");
            let to = params_map.get("to").and_then(|v| v.as_str()).unwrap_or("1.29.0");
            let output = run_talosctl(&["upgrade-k8s", "--from", from, "--to", to]);
            Some(output.map(|_| json!({"status": "k8s upgrade initiated"})))
        }
        _ => None
    }
}

// Handle configuration and etcd methods
fn handle_config_etcd_methods(method: &str, params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "apply_config" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            let file = params_map.get("file").and_then(|v| v.as_str()).ok_or(anyhow!("Missing file param"));
            match (node, file) {
                (Ok(node), Ok(file)) => {
                    let output = run_talosctl(&["--nodes", node, "apply-config", "--file", file]);
                    Some(output.map(|_| json!({"status": "config applied"})))
                }
                (Err(e), _) | (_, Err(e)) => Some(Err(e))
            }
        }
        "validate_config" => {
            let config = params_map.get("config").and_then(|v| v.as_str()).ok_or(anyhow!("Missing config param"));
            let mode = params_map.get("mode").and_then(|v| v.as_str()).unwrap_or("container");
            match config {
                Ok(config) => {
                    let output = run_talosctl(&["validate", "--config", config, "--mode", mode]);
                    Some(output.map(|out| json!({"validation": out})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "get_etcd_status" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "etcd", "status"]);
                    Some(output.map(|out| json!({"etcd_status": out})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "get_etcd_members" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "etcd", "members"]);
                    Some(output.map(|out| json!({"etcd_members": out})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "defrag_etcd" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "etcd", "defrag"]);
                    Some(output.map(|_| json!({"status": "etcd defragmented"})))
                }
                Err(e) => Some(Err(e))
            }
        }
        "bootstrap_etcd" => {
            let node = params_map.get("node").and_then(|v| v.as_str()).ok_or(anyhow!("Missing node param"));
            match node {
                Ok(node) => {
                    let output = run_talosctl(&["--nodes", node, "bootstrap"]);
                    Some(output.map(|_| json!({"status": "etcd bootstrapped"})))
                }
                Err(e) => Some(Err(e))
            }
        }
        _ => None
    }
}


// Handler for each method (following grok.md specification).
fn handle_method(method: &str, params: Option<&Value>) -> Option<Result<Value>> {
    let params_map = extract_params(params);

    // Try MCP protocol methods FIRST (ping, initialize, tools/list, etc.)
    if let Some(result) = handle_mcp_protocol_methods(method, &params_map) {
        return Some(result); // Found a matching MCP method
    }

    // Special handling for notifications that should return None
    if method.starts_with("notifications/") {
        return None; // Notifications should not have responses
    }

    // Try system inspection methods
    if let Some(result) = handle_system_inspection_methods(method, &params_map) {
        return Some(result);
    }

    // Try file operations methods
    if let Some(result) = handle_file_operations_methods(method, &params_map) {
        return Some(result);
    }

    // Try network operations methods
    if let Some(result) = handle_network_operations_methods(method, &params_map) {
        return Some(result);
    }

    // Try service and logging methods
    if let Some(result) = handle_service_log_methods(method, &params_map) {
        return Some(result);
    }

    // Try storage and hardware methods
    if let Some(result) = handle_storage_hardware_methods(method, &params_map) {
        return Some(result);
    }

    // Try core cluster methods
    let result = handle_core_cluster_methods(method, &params_map);
    if result.is_some() {
        return result;
    }

    // Try node management methods
    if let Some(result) = handle_node_management_methods(method, &params_map) {
        return Some(result);
    }

    // Try config/etcd methods
    if let Some(result) = handle_config_etcd_methods(method, &params_map) {
        return Some(result);
    }


    Some(Err(anyhow!("Unknown method: {}", method)))
}

// Main async RPC loop over stdio (from grok.md specification).
async fn rpc_loop() -> Result<()> {
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines.next_line().await? {
        let request: RpcRequest = serde_json::from_str(&line).context("Invalid JSON request")?;

        let result = handle_method(&request.method, request.params.as_ref());
        if let Some(method_result) = result {
            let resp_json = match method_result {
                Ok(res) => {
                    let response = RpcSuccessResponse {
                        jsonrpc: "2.0".to_string(),
                        result: res,
                        id: request.id,
                    };
                    serde_json::to_string(&response)?
                },
                Err(err) => {
                    let response = RpcErrorResponse {
                        jsonrpc: "2.0".to_string(),
                        error: RpcError {
                            code: -32600,
                            message: err.to_string(),
                            data: None,
                        },
                        id: request.id,
                    };
                    serde_json::to_string(&response)?
                },
            };
            stdout.write_all((resp_json + "\n").as_bytes()).await?;
            stdout.flush().await?;
        }
        // If result is None, it's a notification - no response should be sent
    }
    Ok(())
}

fn main() -> Result<()> {
    let rt = Runtime::new()?;
    rt.block_on(rpc_loop())?;

    Ok(())
}