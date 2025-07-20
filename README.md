# Talos MCP Server

A **Model Context Protocol (MCP) server** that provides comprehensive access to Talos OS cluster management capabilities. This server exposes 37 powerful tools for monitoring, managing, and interacting with Talos OS nodes through a standardized MCP interface.

## 🚀 Features

### **System Inspection & Monitoring**
- **Container Management**: List and monitor containers with Kubernetes namespace support
- **Resource Monitoring**: CPU, memory usage, and detailed statistics
- **Process Management**: List and sort running processes
- **Hardware Information**: Disk details, network interfaces, and system resources

### **Network Operations**
- **Interface Management**: Detailed network interface information with multiple output formats
- **Routing**: Complete routing table access with advanced filtering
- **Connectivity**: Network statistics, packet capture, and diagnostics

### **Cluster Management**
- **Health Monitoring**: Comprehensive cluster health checks with configurable parameters
- **Node Operations**: Reboot, shutdown, reset, and upgrade capabilities
- **Time Synchronization**: NTP server verification and time management

### **Storage & File Operations**
- **File System**: Advanced directory listing with filtering, recursion, and multiple formats
- **Disk Management**: Detailed disk information with YAML/JSON output support
- **File Operations**: Read, copy, and manage files on Talos nodes

### **Service & Logging**
- **Service Control**: Start, stop, restart, and monitor system services
- **Log Access**: Retrieve service logs with tail support and Kubernetes integration
- **System Events**: Access kernel messages and system events

### **Configuration & etcd**
- **Configuration Management**: Apply and validate Talos configurations
- **etcd Operations**: Status monitoring, member management, and maintenance

## 📋 Available Tools

| Category | Tool | Enhanced Features |
|----------|------|-------------------|
| **System Monitoring** | `containers` | ✅ `--kubernetes` namespace support |
| | `stats` | ✅ `--kubernetes` namespace support |
| | `get_processes` | ✅ `--sort` by cpu/rss |
| | `memory_verbose` | Detailed memory information |
| | `get_cpu_memory_usage` | Combined CPU/memory stats |
| **File Operations** | `list` | ✅ `--long`, `--humanize`, `--recurse`, `--depth`, `--type` filters |
| | `read` | File content access |
| | `copy` | File transfer operations |
| | `get_usage` | Disk usage information |
| | `get_mounts` | Filesystem mount details |
| **Network** | `interfaces` | ✅ `--namespace`, `--output` (table/json/yaml) |
| | `routes` | ✅ `--namespace`, `--output` (table/json/yaml) |
| | `get_netstat` | Network connection statistics |
| | `capture_packets` | Network packet capture |
| | `get_network_io_cgroups` | Network I/O statistics |
| | `list_network_interfaces` | Legacy interface listing |
| **Services & Logs** | `dmesg` | ✅ Fixed parameter validation |
| | `service` | Service management operations |
| | `restart` | Service restart functionality |
| | `get_logs` | ✅ `--tail` count, `--kubernetes` support |
| | `get_events` | System event monitoring |
| **Storage** | `disks` | ✅ `--namespace`, `--output` (table/json/yaml) |
| | `list_disks` | Legacy disk listing |
| **Cluster Management** | `get_health` | ✅ Enhanced cluster topology support |
| | `get_version` | ✅ `--short` compact format |
| | `get_time` | ✅ `--check` NTP verification, required node parameter |
| **Node Management** | `reboot_node` | Safe node reboot |
| | `shutdown_node` | Graceful node shutdown |
| | `reset_node` | Factory reset operations |
| | `upgrade_node` | Node image upgrades |
| | `upgrade_k8s` | Kubernetes version upgrades |
| **Configuration** | `apply_config` | Configuration deployment |
| | `validate_config` | Configuration validation |
| **etcd** | `get_etcd_status` | etcd cluster status |
| | `get_etcd_members` | Member information |
| | `bootstrap_etcd` | Cluster bootstrapping |
| | `defrag_etcd` | Database defragmentation |

## 🔧 Installation & Setup

### **Prerequisites**
- Rust (latest stable)
- `talosctl` CLI tool installed and configured
- `TALOSCONFIG` environment variable set

### **Build**
```bash
cargo build --release
```

### **Configuration**
Ensure your `TALOSCONFIG` environment variable points to your Talos configuration:
```bash
export TALOSCONFIG=/path/to/your/talosconfig
```

### **Running**
The server communicates over stdio using the MCP protocol:
```bash
./target/release/talos-mcp-server
```

## 📖 Usage Examples

### **Enhanced List Operations**
```json
{
  "method": "list",
  "params": {
    "node": "192.168.1.77",
    "path": "/opt",
    "long": true,
    "humanize": true,
    "recurse": true,
    "type": ["d"]
  }
}
```

### **Network Interface Details (JSON)**
```json
{
  "method": "interfaces",
  "params": {
    "node": "192.168.1.77",
    "output": "json"
  }
}
```

### **Container Monitoring (Kubernetes)**
```json
{
  "method": "containers",
  "params": {
    "node": "192.168.1.77",
    "kubernetes": true
  }
}
```

### **Time Synchronization Check**
```json
{
  "method": "get_time",
  "params": {
    "node": "192.168.1.77",
    "check": "pool.ntp.org"
  }
}
```

### **Service Logs with Tail**
```json
{
  "method": "get_logs",
  "params": {
    "node": "192.168.1.77",
    "service": "kubelet",
    "tail": 100,
    "kubernetes": true
  }
}
```

## ✨ Enhanced Features

### **🔧 Parameter Enhancements**
- **Multiple Output Formats**: Table, JSON, and YAML support for `get` commands
- **Advanced Filtering**: File type filtering, depth control, and sorting options
- **Namespace Support**: Kubernetes and system namespace separation
- **Validation**: Required parameter enforcement with clear error messages

### **📊 Response Enhancements**
All enhanced commands include metadata fields for better tracking:
```json
{
  "interfaces": "...",
  "namespace": "network",
  "output_format": "json"
}
```

### **🛡️ Error Handling**
- Schema-level parameter validation
- Clear error messages for missing required fields
- Graceful handling of command failures

## 🏗️ Architecture

### **Core Components**
- **`main.rs`**: MCP protocol handling and command routing
- **`tools.rs`**: Tool schema definitions and parameter validation
- **Command Handlers**: Organized by functional category (system, network, storage, etc.)

### **Tool Categories**
1. **System Inspection**: `handle_system_inspection_methods()`
2. **File Operations**: `handle_file_operations_methods()`
3. **Network Operations**: `handle_network_operations_methods()`
4. **Service & Logging**: `handle_service_log_methods()`
5. **Storage & Hardware**: `handle_storage_hardware_methods()`
6. **Core Cluster**: `handle_core_cluster_methods()`
7. **Node Management**: `handle_node_management_methods()`
8. **Configuration & etcd**: `handle_config_etcd_methods()`

## 🚀 Development

### **Adding New Tools**
1. Define schema in `tools.rs`
2. Add handler method in appropriate category
3. Implement command logic with parameter processing
4. Add to tool list in `get_all_tool_schemas()`

### **Testing**
All enhanced features include comprehensive parameter validation and response formatting. Test using MCP-compatible clients or the provided tool interfaces.

### **CI/CD**
GitHub Actions workflow provides:
- Multi-platform builds (Linux, macOS, Windows)
- Docker container builds
- Security scanning and testing
- Automated releases

## 📚 MCP Integration

This server implements the **Model Context Protocol (MCP)** specification, making it compatible with:
- Claude Desktop
- Custom MCP clients
- AI assistants and development tools

### **Protocol Features**
- JSON-RPC 2.0 over stdio
- Tool discovery and schema validation
- Structured parameter passing
- Rich response formatting

## 🤝 Contributing

Contributions welcome! Please ensure:
- All new tools include comprehensive schemas
- Enhanced response formats with metadata
- Proper error handling and validation
- Documentation updates

## 📄 License

This project follows standard open-source practices. Please ensure compliance with Talos OS and related component licenses.

---

**🎯 Perfect for:** DevOps automation, cluster monitoring, system administration, and AI-assisted Talos management workflows.