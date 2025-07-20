use serde_json::{json, Value};

/// Get all tool schemas with descriptions and parameter definitions
pub fn get_all_tool_schemas() -> Value {
    json!({
        "tools": [
            // System inspection and monitoring
            get_containers_schema(),
            get_stats_schema(),
            get_processes_schema(),
            get_memory_verbose_schema(),
            get_cpu_memory_usage_schema(),

            // File system operations
            get_list_schema(),
            get_read_schema(),
            get_copy_schema(),
            get_usage_schema(),
            get_mounts_schema(),

            // Network operations
            get_interfaces_schema(),
            get_routes_schema(),
            get_netstat_schema(),
            get_capture_packets_schema(),
            get_network_io_cgroups_schema(),
            get_list_network_interfaces_schema(),

            // Service and logging
            get_dmesg_schema(),
            get_service_schema(),
            get_restart_schema(),
            get_logs_schema(),
            get_events_schema(),

            // Storage and hardware
            get_disks_schema(),
            get_list_disks_schema(),

            // Core cluster management
            get_health_schema(),
            get_version_schema(),
            get_time_schema(),

            // Node management
            get_reboot_node_schema(),
            get_shutdown_node_schema(),
            get_reset_node_schema(),
            get_upgrade_node_schema(),
            get_upgrade_k8s_schema(),

            // Configuration management
            get_apply_config_schema(),
            get_validate_config_schema(),

            // etcd management
            get_etcd_status_schema(),
            get_etcd_members_schema(),
            get_bootstrap_etcd_schema(),
            get_defrag_etcd_schema()
        ]
    })
}

// System inspection and monitoring schemas
fn get_containers_schema() -> Value {
    json!({
        "name": "containers",
        "description": "List running containers on a Talos node with their current status",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                },
                "kubernetes": {
                    "type": "boolean",
                    "description": "Use the k8s.io containerd namespace to list Kubernetes containers (defaults to false)",
                    "default": false
                }
            },
            "required": ["node"]
        }
    })
}

fn get_stats_schema() -> Value {
    json!({
        "name": "stats",
        "description": "Get resource usage statistics (CPU, memory) for containers on a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                },
                "kubernetes": {
                    "type": "boolean",
                    "description": "Use the k8s.io containerd namespace to get Kubernetes containers stats (defaults to false)",
                    "default": false
                }
            },
            "required": ["node"]
        }
    })
}

fn get_memory_verbose_schema() -> Value {
    json!({
        "name": "memory_verbose",
        "description": "Get detailed memory usage information from a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_list_schema() -> Value {
    json!({
        "name": "list",
        "description": "List files and directories at a specified path on a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                },
                "path": {
                    "type": "string",
                    "description": "Directory path to list (defaults to root /)",
                    "default": "/"
                },
                "long": {
                    "type": "boolean",
                    "description": "Display additional file details",
                    "default": false
                },
                "humanize": {
                    "type": "boolean",
                    "description": "Humanize size and time in the output",
                    "default": false
                },
                "recurse": {
                    "type": "boolean",
                    "description": "Recurse into subdirectories",
                    "default": false
                },
                "depth": {
                    "type": "integer",
                    "description": "Maximum recursion depth (defaults to 1)",
                    "minimum": 1,
                    "default": 1
                },
                "type": {
                    "type": "array",
                    "description": "Filter by specified file types",
                    "items": {
                        "type": "string",
                        "enum": ["f", "d", "l", "L"]
                    }
                }
            },
            "required": ["node"]
        }
    })
}

fn get_read_schema() -> Value {
    json!({
        "name": "read",
        "description": "Read the contents of a file on a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                },
                "path": {
                    "type": "string",
                    "description": "Full path to the file to read"
                }
            },
            "required": ["node", "path"]
        }
    })
}

fn get_interfaces_schema() -> Value {
    json!({
        "name": "interfaces",
        "description": "Get detailed network interface information including addresses and links",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                },
                "namespace": {
                    "type": "string",
                    "description": "Resource namespace (default is to use default namespace per resource)"
                },
                "output": {
                    "type": "string",
                    "description": "Output mode (default: table)",
                    "enum": ["json", "table", "yaml", "jsonpath"],
                    "default": "table"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_routes_schema() -> Value {
    json!({
        "name": "routes",
        "description": "Get network routing table information for a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                },
                "namespace": {
                    "type": "string",
                    "description": "Resource namespace (default is to use default namespace per resource)"
                },
                "output": {
                    "type": "string",
                    "description": "Output mode (default: table)",
                    "enum": ["json", "table", "yaml", "jsonpath"],
                    "default": "table"
                }
            },
            "required": ["node"]
        }
    })
}

// Service and logging schemas
fn get_dmesg_schema() -> Value {
    json!({
        "name": "dmesg",
        "description": "Get kernel ring buffer messages (system logs) from a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_service_schema() -> Value {
    json!({
        "name": "service",
        "description": "Manage services on a Talos node (get status, start, stop, restart)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                },
                "service": {
                    "type": "string",
                    "description": "Name of the service to manage (e.g., kubelet, etcd, containerd)"
                },
                "action": {
                    "type": "string",
                    "description": "Action to perform on the service (defaults to 'status')",
                    "enum": ["status", "start", "stop", "restart"],
                    "default": "status"
                }
            },
            "required": ["node", "service"]
        }
    })
}

fn get_restart_schema() -> Value {
    json!({
        "name": "restart",
        "description": "Restart a specific service on a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node"
                },
                "service": {
                    "type": "string",
                    "description": "Name of the service to restart (e.g., kubelet, etcd, containerd)"
                }
            },
            "required": ["node", "service"]
        }
    })
}

// File system operation schemas
fn get_copy_schema() -> Value {
    json!({
        "name": "copy",
        "description": "Copy files to/from a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node"
                },
                "source": {
                    "type": "string",
                    "description": "Source file path (local or remote)"
                },
                "destination": {
                    "type": "string",
                    "description": "Destination file path (local or remote)"
                }
            },
            "required": ["node", "source", "destination"]
        }
    })
}

// Storage and hardware schemas
fn get_disks_schema() -> Value {
    json!({
        "name": "disks",
        "description": "Get detailed disk information from a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                },
                "namespace": {
                    "type": "string",
                    "description": "Resource namespace (default is to use default namespace per resource)"
                },
                "output": {
                    "type": "string",
                    "description": "Output mode (default: table)",
                    "enum": ["json", "table", "yaml", "jsonpath"],
                    "default": "table"
                }
            },
            "required": ["node"]
        }
    })
}

// Network operation schemas
// Network operation schemas

// Core cluster management schemas
fn get_health_schema() -> Value {
    json!({
        "name": "get_health",
        "description": "Check the health status of the Talos cluster",
        "inputSchema": {
            "type": "object",
            "properties": {
                "control_planes": {
                    "type": "array",
                    "description": "Array of IP addresses or hostnames of control plane nodes (defaults to [192.168.1.77])",
                    "items": {"type": "string"},
                    "default": ["192.168.1.77"]
                },
                "worker_nodes": {
                    "type": "array",
                    "description": "Array of IP addresses or hostnames of worker nodes",
                    "items": {"type": "string"}
                },
                "init_node": {
                    "type": "string",
                    "description": "IP address or hostname of the init node"
                },
                "timeout": {
                    "type": "string",
                    "description": "Timeout duration for health check (defaults to 120s)",
                    "default": "120s"
                },
                "run_e2e": {
                    "type": "boolean",
                    "description": "Run Kubernetes e2e test (defaults to false)",
                    "default": false
                },
                "k8s_endpoint": {
                    "type": "string",
                    "description": "Use endpoint instead of kubeconfig default"
                },
                "server": {
                    "type": "boolean",
                    "description": "Run server-side check (defaults to true)",
                    "default": true
                }
            }
        }
    })
}

fn get_version_schema() -> Value {
    json!({
        "name": "get_version",
        "description": "Get Talos client version information",
        "inputSchema": {
            "type": "object",
            "properties": {
                "short": {
                    "type": "boolean",
                    "description": "Print the short version (defaults to false)",
                    "default": false
                }
            }
        }
    })
}

fn get_processes_schema() -> Value {
    json!({
        "name": "get_processes",
        "description": "List running processes on a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                },
                "sort": {
                    "type": "string",
                    "description": "Column to sort output by (defaults to 'rss')",
                    "enum": ["rss", "cpu"],
                    "default": "rss"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_logs_schema() -> Value {
    json!({
        "name": "get_logs",
        "description": "Get service logs from a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                },
                "service": {
                    "type": "string",
                    "description": "Name of the service to get logs for (e.g., kubelet, etcd)"
                },
                "tail": {
                    "type": "integer",
                    "description": "Number of lines to show from the end of the logs (e.g., 100)",
                    "minimum": 1
                },
                "kubernetes": {
                    "type": "boolean",
                    "description": "Use the k8s.io containerd namespace to access Kubernetes containers (defaults to false)",
                    "default": false
                }
            },
            "required": ["node", "service"]
        }
    })
}

fn get_usage_schema() -> Value {
    json!({
        "name": "get_usage",
        "description": "Get disk usage information for a path on a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                },
                "path": {
                    "type": "string",
                    "description": "Path to check disk usage for (defaults to root /)",
                    "default": "/"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_mounts_schema() -> Value {
    json!({
        "name": "get_mounts",
        "description": "Get filesystem mount information from a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_time_schema() -> Value {
    json!({
        "name": "get_time",
        "description": "Get current time from a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                },
                "check": {
                    "type": "string",
                    "description": "Check server time against specified NTP server (e.g., 'pool.ntp.org')"
                }
            },
            "required": ["node"]
        }
    })
}

// Node management schemas
fn get_reboot_node_schema() -> Value {
    json!({
        "name": "reboot_node",
        "description": "Reboot a Talos node (DESTRUCTIVE OPERATION)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to reboot"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_shutdown_node_schema() -> Value {
    json!({
        "name": "shutdown_node",
        "description": "Shutdown a Talos node (DESTRUCTIVE OPERATION)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to shutdown"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_reset_node_schema() -> Value {
    json!({
        "name": "reset_node",
        "description": "Reset a Talos node to factory defaults (DESTRUCTIVE OPERATION)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to reset"
                }
            },
            "required": ["node"]
        }
    })
}

// Configuration schemas
fn get_apply_config_schema() -> Value {
    json!({
        "name": "apply_config",
        "description": "Apply a configuration file to a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to configure"
                },
                "file": {
                    "type": "string",
                    "description": "Path to the configuration file to apply"
                }
            },
            "required": ["node", "file"]
        }
    })
}

fn get_validate_config_schema() -> Value {
    json!({
        "name": "validate_config",
        "description": "Validate a Talos configuration file",
        "inputSchema": {
            "type": "object",
            "properties": {
                "config": {
                    "type": "string",
                    "description": "Path to the configuration file to validate"
                },
                "mode": {
                    "type": "string",
                    "description": "Validation mode (defaults to 'container')",
                    "default": "container"
                }
            },
            "required": ["config"]
        }
    })
}

// etcd management schemas
fn get_etcd_status_schema() -> Value {
    json!({
        "name": "get_etcd_status",
        "description": "Get etcd cluster status from a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_etcd_members_schema() -> Value {
    json!({
        "name": "get_etcd_members",
        "description": "Get etcd cluster member information from a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_bootstrap_etcd_schema() -> Value {
    json!({
        "name": "bootstrap_etcd",
        "description": "Bootstrap etcd cluster on a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to bootstrap"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_defrag_etcd_schema() -> Value {
    json!({
        "name": "defrag_etcd",
        "description": "Defragment etcd database on a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to defragment"
                }
            },
            "required": ["node"]
        }
    })
}

// Network monitoring schemas
fn get_netstat_schema() -> Value {
    json!({
        "name": "get_netstat",
        "description": "Get network connection statistics from a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_capture_packets_schema() -> Value {
    json!({
        "name": "capture_packets",
        "description": "Capture network packets on a Talos node interface",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to capture from"
                },
                "interface": {
                    "type": "string",
                    "description": "Network interface to capture from (defaults to eth0)",
                    "default": "eth0"
                },
                "duration": {
                    "type": "string",
                    "description": "Duration to capture packets (defaults to 10s)",
                    "default": "10s"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_network_io_cgroups_schema() -> Value {
    json!({
        "name": "get_network_io_cgroups",
        "description": "Get network I/O cgroup statistics from a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_events_schema() -> Value {
    json!({
        "name": "get_events",
        "description": "Get system events from a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                }
            },
            "required": ["node"]
        }
    })
}

// Upgrade operation schemas
fn get_upgrade_node_schema() -> Value {
    json!({
        "name": "upgrade_node",
        "description": "Upgrade a Talos node to a new image version",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to upgrade"
                },
                "image": {
                    "type": "string",
                    "description": "Container image to upgrade to (defaults to latest installer)",
                    "default": "ghcr.io/siderolabs/installer:latest"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_upgrade_k8s_schema() -> Value {
    json!({
        "name": "upgrade_k8s",
        "description": "Upgrade Kubernetes cluster version",
        "inputSchema": {
            "type": "object",
            "properties": {
                "from": {
                    "type": "string",
                    "description": "Current Kubernetes version (defaults to 1.28.0)",
                    "default": "1.28.0"
                },
                "to": {
                    "type": "string",
                    "description": "Target Kubernetes version (defaults to 1.29.0)",
                    "default": "1.29.0"
                }
            }
        }
    })
}

// Legacy method schemas
fn get_list_disks_schema() -> Value {
    json!({
        "name": "list_disks",
        "description": "List disk devices on a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_list_network_interfaces_schema() -> Value {
    json!({
        "name": "list_network_interfaces",
        "description": "List network interfaces on a Talos node (legacy method)",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                }
            },
            "required": ["node"]
        }
    })
}

fn get_cpu_memory_usage_schema() -> Value {
    json!({
        "name": "get_cpu_memory_usage",
        "description": "Get CPU and memory usage statistics from a Talos node",
        "inputSchema": {
            "type": "object",
            "properties": {
                "node": {
                    "type": "string",
                    "description": "IP address or hostname of the Talos node to query"
                }
            },
            "required": ["node"]
        }
    })
}