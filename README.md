# StudioLink

**Advanced Roblox Studio MCP Server — 36 tools for professional game development with AI**

StudioLink is a high-performance [Model Context Protocol](https://modelcontextprotocol.io/) (MCP) server built in Rust that connects AI assistants (Claude, Cursor, etc.) directly to Roblox Studio. It provides 36 specialized tools covering code execution, play testing, security auditing, performance profiling, DataStore debugging, and much more.

## Why StudioLink?

Roblox's official MCP server provides 6 basic tools. StudioLink gives you **36 tools** with features like:

- Execute code in **Server context during play mode** (not just Edit mode)
- Multi-instance support — manage multiple Studio windows simultaneously
- Security scanning, memory leak detection, dependency mapping
- DataStore CRUD with live production data
- ScriptProfiler integration with CPU hotspot analysis
- Full TestEZ test framework support
- Animation conflict detection, UI analysis, network traffic monitoring

## Tools

### Core (6 tools)
| Tool | Description |
|------|-------------|
| `run_code` | Execute Luau code in Studio (Edit or Server context) |
| `insert_model` | Search and insert models from Creator Store |
| `get_console_output` | Read Studio output console |
| `start_stop_play` | Start/stop play mode via StudioTestService |
| `run_script_in_play_mode` | Run scripts in play mode with timeout |
| `get_studio_mode` | Get current Studio mode (edit/play/run) |

### Session Management (3 tools)
| Tool | Description |
|------|-------------|
| `get_active_session` | Get current active Studio session info |
| `list_sessions` | List all connected Studio instances |
| `switch_session` | Switch between Studio instances |

### DataStore Debugging (5 tools)
| Tool | Description |
|------|-------------|
| `datastore_list` | List all DataStores in the experience |
| `datastore_get` | Read a specific key's value |
| `datastore_set` | Write a value to a key |
| `datastore_delete` | Delete a key |
| `datastore_scan` | Scan all keys with pagination |

### Performance Profiling (3 tools)
| Tool | Description |
|------|-------------|
| `profile_start` | Start ScriptProfiler (configurable frequency) |
| `profile_stop` | Stop profiler, get raw data |
| `profile_analyze` | Analyze CPU hotspots with optimization suggestions |

### Place Versioning (3 tools)
| Tool | Description |
|------|-------------|
| `snapshot_take` | Capture full place state (instances, properties, scripts) |
| `snapshot_compare` | Diff two snapshots |
| `snapshot_list` | List saved snapshots |

### Test Framework (3 tools)
| Tool | Description |
|------|-------------|
| `test_run` | Run TestEZ test suites |
| `test_create` | Auto-generate test templates for a script |
| `test_report` | Get detailed test results |

### Security Auditor (2 tools)
| Tool | Description |
|------|-------------|
| `security_scan` | Scan for vulnerabilities (unvalidated Remotes, client trust, data exposure) |
| `security_report` | Formatted report with risk levels and remediation steps |

### Code Analysis (3 tools)
| Tool | Description |
|------|-------------|
| `dependency_map` | Map require() chains, detect circular deps and dead code |
| `memory_scan` | Detect memory leaks (Connections, Instances, RunService bindings) |
| `lint_scripts` | Find deprecated APIs, anti-patterns, naming issues |

### Animation (3 tools)
| Tool | Description |
|------|-------------|
| `animation_list` | List all animations with IDs, durations, priorities |
| `animation_inspect` | Get detailed keyframe info for an animation |
| `animation_conflicts` | Detect conflicting animations on same body parts |

### Network (2 tools)
| Tool | Description |
|------|-------------|
| `network_monitor_start` | Start monitoring RemoteEvent/Function traffic |
| `network_monitor_stop` | Stop and get traffic report (frequency, bandwidth, spam) |

### UI Inspector (2 tools)
| Tool | Description |
|------|-------------|
| `ui_tree` | Get full GUI hierarchy with sizes and positions |
| `ui_analyze` | Detect overlapping UI, off-screen elements, ZIndex conflicts |

### Documentation (1 tool)
| Tool | Description |
|------|-------------|
| `docs_generate` | Auto-generate Markdown docs for all ModuleScripts |

## Architecture

```
┌─────────────┐     stdio/JSON-RPC      ┌──────────────┐     HTTP long poll     ┌──────────────┐
│  AI Client   │◄──────────────────────►│  StudioLink   │◄────────────────────►│ Studio Plugin │
│ (Claude etc) │        MCP              │  (Rust)       │    localhost:34872    │   (Luau)      │
└─────────────┘                          └──────────────┘                       └──────────────┘
```

During play mode, the plugin registers **two sessions**:
- **Edit session** — runs the full tool suite in Edit context
- **Play Server session** — runs `run_code` in Server context with access to runtime game state (Players, spawned objects, Knit services, etc.)

Use `switch_session` to toggle between them.

## Installation

### Option 1: Download Release

Download the latest release from [Releases](https://github.com/realapeiron/StudioLink/releases):
- **macOS** — Universal binary (Apple Silicon + Intel)
- **Windows** — x86_64 exe
- **Linux** — x86_64 binary
- **Plugin** — StudioLink.rbxm

### Option 2: Build from Source

**Prerequisites:** [Rust](https://rustup.rs/) 1.75+, [Rojo](https://rojo.space/) 7.x, Roblox Studio

```bash
git clone https://github.com/realapeiron/StudioLink.git
cd StudioLink

# Build the server
cargo build --release

# Build the plugin
cd plugin
rojo build -o StudioLink.rbxm
```

### Install the Plugin

Copy `StudioLink.rbxm` to your Roblox Studio plugins folder:
- **Windows:** `%LOCALAPPDATA%/Roblox/Plugins/`
- **macOS:** `~/Documents/Roblox/Plugins/`

### Configure Your MCP Client

**Claude Desktop / Claude Code:**
```json
{
  "mcpServers": {
    "studiolink": {
      "command": "/path/to/studiolink"
    }
  }
}
```

**With custom port:**
```json
{
  "mcpServers": {
    "studiolink": {
      "command": "/path/to/studiolink",
      "args": ["--port", "34872"]
    }
  }
}
```

**Proxy mode** (connect to an already-running StudioLink server):
```json
{
  "mcpServers": {
    "studiolink": {
      "command": "/path/to/studiolink",
      "args": ["--proxy", "http://127.0.0.1:34872"]
    }
  }
}
```

## Roblox Studio Setup

1. Enable **HTTP Requests** in Game Settings → Security
2. Enable **Studio Access to API Services** (for DataStore tools)
3. Install the StudioLink plugin
4. Start the StudioLink server
5. The plugin auto-connects and registers the session

## Play Mode Server Context

StudioLink can execute code in the **Server context** during play mode — something no other MCP server can do. This enables:

- Accessing runtime game state (Players, Characters, spawned objects)
- Calling Knit/other framework services directly
- Reading/writing to `_G`, DataStores, and server-side state
- Simulating game actions (collecting items, triggering events)

**How it works:** When play starts, the plugin registers a second "Play Server" session. The AI switches to it via `switch_session` and uses `run_code` to execute Luau in Server context via dynamic Script injection.

## Contributing

Contributions welcome! Please open an issue or PR.

## License

MIT License — see [LICENSE](LICENSE) for details.
