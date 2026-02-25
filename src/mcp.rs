use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use crate::tools;

// ═══════════════════════════════════════════════════════
// PARAMETER STRUCTS
// ═══════════════════════════════════════════════════════

// --- Core ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RunCodeParams {
    /// Luau code to execute in Roblox Studio
    pub command: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct InsertModelParams {
    /// Search query for the Roblox Creator Store
    pub query: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct StartStopPlayParams {
    /// Mode: 'start_play', 'stop', or 'run_server'
    pub mode: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RunScriptInPlayModeParams {
    /// Luau code to run in play mode
    pub code: String,
    /// Mode: 'start_play' or 'run_server'
    pub mode: String,
    /// Timeout in seconds (default: 100)
    pub timeout: Option<u64>,
}

// --- DataStore ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DataStoreGetParams {
    /// Name of the DataStore
    pub store_name: String,
    /// Key to read
    pub key: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DataStoreSetParams {
    /// Name of the DataStore
    pub store_name: String,
    /// Key to write
    pub key: String,
    /// Value to set (any JSON value)
    pub value: Value,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DataStoreDeleteParams {
    /// Name of the DataStore
    pub store_name: String,
    /// Key to delete
    pub key: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DataStoreScanParams {
    /// Name of the DataStore
    pub store_name: String,
    /// Number of keys per page
    pub page_size: Option<u32>,
    /// Pagination cursor from previous scan
    pub cursor: Option<String>,
}

// --- Profiler ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ProfileStartParams {
    /// Sampling frequency in Hz (default: 1000)
    pub frequency: Option<u32>,
}

// --- Diffing ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SnapshotTakeParams {
    /// Optional name for the snapshot
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SnapshotCompareParams {
    /// ID or name of the first snapshot
    pub snapshot_a: String,
    /// ID or name of the second snapshot
    pub snapshot_b: String,
}

// --- Testing ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct TestRunParams {
    /// Optional path to run tests for a specific module
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct TestCreateParams {
    /// Path to the script or ModuleScript to generate tests for
    pub target_path: String,
}

// --- Linter ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LintScriptsParams {
    /// Optional path to limit analysis scope
    pub path: Option<String>,
}

// --- Animation ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AnimationInspectParams {
    /// Animation asset ID to inspect
    pub animation_id: String,
}

// --- Docs ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DocsGenerateParams {
    /// Optional path to limit documentation scope
    pub path: Option<String>,
}

// --- Workspace ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct WorkspaceAnalyzeParams {
    /// Optional path to limit analysis scope (e.g. "ServerScriptService")
    pub path: Option<String>,
}

// --- Instance Management ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetFileTreeParams {
    /// Optional path to get tree for (e.g. "Workspace.MyModel"). If empty, returns all services.
    pub path: Option<String>,
    /// Maximum depth to traverse (default: 10)
    pub depth: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetInstancePropertiesParams {
    /// Dot-separated path to the instance (e.g. "Workspace.Part")
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SetPropertyParams {
    /// Dot-separated path to the instance
    pub path: String,
    /// Property name to set
    pub property: String,
    /// Value to set
    pub value: Value,
    /// Optional value type hint: "string", "number", "boolean", "Vector3", "Color3", "UDim2", "BrickColor", "Enum"
    #[serde(rename = "valueType")]
    pub value_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MassSetPropertyParams {
    /// Array of dot-separated paths to instances
    pub paths: Vec<String>,
    /// Property name to set
    pub property: String,
    /// Value to set
    pub value: Value,
    /// Optional value type hint
    #[serde(rename = "valueType")]
    pub value_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateInstanceParams {
    /// Roblox class name (e.g. "Part", "Script", "Folder")
    #[serde(rename = "className")]
    pub class_name: String,
    /// Dot-separated path to the parent instance (default: Workspace)
    #[serde(rename = "parentPath")]
    pub parent_path: Option<String>,
    /// Optional properties to set on the new instance
    pub properties: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DeleteInstanceParams {
    /// Dot-separated path to the instance to delete
    pub path: String,
}

// --- Script Tools ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetScriptSourceParams {
    /// Dot-separated path to the script (e.g. "ServerScriptService.MyScript")
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SetScriptSourceParams {
    /// Dot-separated path to the script
    pub path: String,
    /// New source code for the script
    pub source: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GrepScriptsParams {
    /// Text pattern to search for in all scripts
    pub pattern: String,
    /// Whether the search is case sensitive (default: true)
    #[serde(rename = "caseSensitive")]
    pub case_sensitive: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SearchObjectsParams {
    /// Search query (name or class to search for)
    pub query: String,
    /// Search mode: "name", "class", or "both" (default: "name")
    #[serde(rename = "searchBy")]
    pub search_by: Option<String>,
}

// --- Session ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SwitchSessionParams {
    /// Session ID to switch to
    pub session_id: String,
}

// ═══════════════════════════════════════════════════════
// MCP SERVER HANDLER
// ═══════════════════════════════════════════════════════

/// StudioLink MCP Server handler — registers and dispatches all 49 tools
#[derive(Clone)]
pub struct StudioLinkMcp {
    pub state: Arc<Mutex<AppState>>,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

impl StudioLinkMcp {
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        let tool_router = Self::tool_router();
        Self { state, tool_router }
    }
}

/// Helper: format tool result as success text
fn ok_text(result: serde_json::Value) -> String {
    result.to_string()
}

/// Helper: format tool result as error text
fn err_text(e: impl std::fmt::Display) -> String {
    format!("Error: {}", e)
}

#[tool_router]
impl StudioLinkMcp {
    // ═══════════════════════════════════════════
    // FAZ 1: CORE TOOLS
    // ═══════════════════════════════════════════

    #[tool(description = "Execute Luau code in Roblox Studio and return the printed output. Can be used to both make changes and retrieve information.")]
    async fn run_code(
        &self,
        params: Parameters<RunCodeParams>,
    ) -> String {
        match tools::core::run_code(&self.state, &params.0.command).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Search and insert a model from the Roblox Creator Store into the workspace.")]
    async fn insert_model(
        &self,
        params: Parameters<InsertModelParams>,
    ) -> String {
        match tools::core::insert_model(&self.state, &params.0.query).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Get the console output from Roblox Studio.")]
    async fn get_console_output(&self) -> String {
        match tools::core::get_console_output(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Start or stop play mode or run the server. Mode must be 'start_play', 'stop', or 'run_server'.")]
    async fn start_stop_play(
        &self,
        params: Parameters<StartStopPlayParams>,
    ) -> String {
        match tools::core::start_stop_play(&self.state, &params.0.mode).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Run a Luau script in play mode with automatic stop after completion or timeout. Returns structured output with logs, errors, and duration.")]
    async fn run_script_in_play_mode(
        &self,
        params: Parameters<RunScriptInPlayModeParams>,
    ) -> String {
        match tools::core::run_script_in_play_mode(
            &self.state, &params.0.code, &params.0.mode, params.0.timeout,
        ).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Get the current Roblox Studio mode: 'start_play', 'run_server', or 'stop'.")]
    async fn get_studio_mode(&self) -> String {
        match tools::core::get_studio_mode(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // FAZ 2: DATASTORE & PROFILING
    // ═══════════════════════════════════════════

    #[tool(description = "List all DataStore names in the current experience. Requires 'Allow Studio Access to API Services' enabled in game settings.")]
    async fn datastore_list(&self) -> String {
        match tools::datastore::datastore_list(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Read a specific key's value from a DataStore.")]
    async fn datastore_get(
        &self,
        params: Parameters<DataStoreGetParams>,
    ) -> String {
        match tools::datastore::datastore_get(&self.state, &params.0.store_name, &params.0.key).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Write a value to a DataStore key. WARNING: This modifies live production data.")]
    async fn datastore_set(
        &self,
        params: Parameters<DataStoreSetParams>,
    ) -> String {
        match tools::datastore::datastore_set(
            &self.state, &params.0.store_name, &params.0.key, params.0.value,
        ).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Delete a key from a DataStore. WARNING: This permanently removes live production data.")]
    async fn datastore_delete(
        &self,
        params: Parameters<DataStoreDeleteParams>,
    ) -> String {
        match tools::datastore::datastore_delete(&self.state, &params.0.store_name, &params.0.key).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Scan and list all keys in a DataStore with pagination support.")]
    async fn datastore_scan(
        &self,
        params: Parameters<DataStoreScanParams>,
    ) -> String {
        match tools::datastore::datastore_scan(
            &self.state, &params.0.store_name, params.0.page_size, params.0.cursor.as_deref(),
        ).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Start the ScriptProfiler to measure CPU time per function. Optional frequency in Hz (default: 1000).")]
    async fn profile_start(
        &self,
        params: Parameters<ProfileStartParams>,
    ) -> String {
        match tools::profiler::profile_start(&self.state, params.0.frequency).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Stop the ScriptProfiler and return raw profiling data.")]
    async fn profile_stop(&self) -> String {
        match tools::profiler::profile_stop(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Analyze profiling data: slowest functions, CPU hotspots, and optimization suggestions.")]
    async fn profile_analyze(&self) -> String {
        match tools::profiler::profile_analyze(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // FAZ 3: DIFFING & TESTING
    // ═══════════════════════════════════════════

    #[tool(description = "Take a snapshot of the current place state (all instances, properties, scripts). Optional name for the snapshot.")]
    async fn snapshot_take(
        &self,
        params: Parameters<SnapshotTakeParams>,
    ) -> String {
        match tools::diffing::snapshot_take(&self.state, params.0.name.as_deref()).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Compare two snapshots and list all differences (added/removed/changed instances and properties).")]
    async fn snapshot_compare(
        &self,
        params: Parameters<SnapshotCompareParams>,
    ) -> String {
        match tools::diffing::snapshot_compare(
            &self.state, &params.0.snapshot_a, &params.0.snapshot_b,
        ).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "List all saved snapshots with names and timestamps.")]
    async fn snapshot_list(&self) -> String {
        match tools::diffing::snapshot_list(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Run TestEZ test suites. Optionally specify a path to run tests for a specific module.")]
    async fn test_run(
        &self,
        params: Parameters<TestRunParams>,
    ) -> String {
        match tools::testing::test_run(&self.state, params.0.path.as_deref()).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Generate a TestEZ test template for a given script or ModuleScript.")]
    async fn test_create(
        &self,
        params: Parameters<TestCreateParams>,
    ) -> String {
        match tools::testing::test_create(&self.state, &params.0.target_path).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Get the detailed results from the last test run.")]
    async fn test_report(&self) -> String {
        match tools::testing::test_report(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // FAZ 4: SECURITY & ANALYSIS
    // ═══════════════════════════════════════════

    #[tool(description = "Scan the entire place for security vulnerabilities: unvalidated RemoteEvents, client trust issues, exposed data, missing rate limiting.")]
    async fn security_scan(&self) -> String {
        match tools::security::security_scan(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Get a formatted security report with risk levels (Critical/High/Medium/Low) and remediation suggestions.")]
    async fn security_report(&self) -> String {
        match tools::security::security_report(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Map all require() chains across the project. Detects circular dependencies, dead code (unrequired modules), and usage statistics.")]
    async fn dependency_map(&self) -> String {
        match tools::dependencies::dependency_map(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Scan for potential memory leaks: undisconnected Connections, undestroyed instances, growing tables, excessive RunService bindings.")]
    async fn memory_scan(&self) -> String {
        match tools::memory::memory_scan(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Analyze scripts for code quality: deprecated APIs, anti-patterns, naming issues, unused variables, missing type annotations.")]
    async fn lint_scripts(
        &self,
        params: Parameters<LintScriptsParams>,
    ) -> String {
        match tools::linter::lint_scripts(&self.state, params.0.path.as_deref()).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // FAZ 5: INSPECTOR TOOLS
    // ═══════════════════════════════════════════

    #[tool(description = "List all animations in the place with their IDs, durations, and priorities.")]
    async fn animation_list(&self) -> String {
        match tools::animation::animation_list(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Get detailed keyframe information for a specific animation.")]
    async fn animation_inspect(
        &self,
        params: Parameters<AnimationInspectParams>,
    ) -> String {
        match tools::animation::animation_inspect(&self.state, &params.0.animation_id).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Detect conflicting animations that affect the same body parts simultaneously.")]
    async fn animation_conflicts(&self) -> String {
        match tools::animation::animation_conflicts(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Start monitoring all RemoteEvent and RemoteFunction traffic (call frequency, data size, spam detection).")]
    async fn network_monitor_start(&self) -> String {
        match tools::network::network_monitor_start(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Stop network monitoring and return a detailed traffic report with per-Remote statistics and bandwidth estimates.")]
    async fn network_monitor_stop(&self) -> String {
        match tools::network::network_monitor_stop(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Get the full GUI hierarchy with sizes and positions.")]
    async fn ui_tree(&self) -> String {
        match tools::ui_inspector::ui_tree(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Detect UI issues: overlapping elements, off-screen UI, mobile touch target sizes, ZIndex conflicts, missing layout components.")]
    async fn ui_analyze(&self) -> String {
        match tools::ui_inspector::ui_analyze(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Auto-generate Markdown documentation for all ModuleScripts: public functions, parameter types, return types, dependencies.")]
    async fn docs_generate(
        &self,
        params: Parameters<DocsGenerateParams>,
    ) -> String {
        match tools::docs::docs_generate(&self.state, params.0.path.as_deref()).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // WORKSPACE ANALYSIS
    // ═══════════════════════════════════════════

    #[tool(description = "Comprehensive workspace analysis: coding style (naming, indent, strict mode, type annotations), architecture (framework, services, folder structure), script statistics, issues (deprecated APIs, security, memory leaks, optimization), dependencies (circular, dead modules), and detected patterns/libraries. Run this first on any new workspace.")]
    async fn workspace_analyze(
        &self,
        params: Parameters<WorkspaceAnalyzeParams>,
    ) -> String {
        match tools::workspace::workspace_analyze(&self.state, params.0.path.as_deref()).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // INSTANCE MANAGEMENT
    // ═══════════════════════════════════════════

    #[tool(description = "Get a hierarchical tree of all instances in the place. Optionally specify a path to focus on a subtree and depth to limit traversal.")]
    async fn get_file_tree(
        &self,
        params: Parameters<GetFileTreeParams>,
    ) -> String {
        match tools::instance::get_file_tree(&self.state, params.0.path.as_deref(), params.0.depth).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Get all properties of an instance at the given path, including class-specific properties (BasePart, GuiObject, Light, etc.), attributes, and tags.")]
    async fn get_instance_properties(
        &self,
        params: Parameters<GetInstancePropertiesParams>,
    ) -> String {
        match tools::instance::get_instance_properties(&self.state, &params.0.path).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Set a single property on an instance. Supports type hints for Vector3, Color3, UDim2, BrickColor, Enum values.")]
    async fn set_property(
        &self,
        params: Parameters<SetPropertyParams>,
    ) -> String {
        match tools::instance::set_property(
            &self.state, &params.0.path, &params.0.property, params.0.value, params.0.value_type.as_deref(),
        ).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Set the same property on multiple instances at once. Provide an array of paths.")]
    async fn mass_set_property(
        &self,
        params: Parameters<MassSetPropertyParams>,
    ) -> String {
        match tools::instance::mass_set_property(
            &self.state, params.0.paths, &params.0.property, params.0.value, params.0.value_type.as_deref(),
        ).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Create a new instance with the given class name under a parent path. Optionally set initial properties.")]
    async fn create_instance(
        &self,
        params: Parameters<CreateInstanceParams>,
    ) -> String {
        match tools::instance::create_instance(
            &self.state, &params.0.class_name, params.0.parent_path.as_deref(), params.0.properties,
        ).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Delete an instance and all its descendants at the given path.")]
    async fn delete_instance(
        &self,
        params: Parameters<DeleteInstanceParams>,
    ) -> String {
        match tools::instance::delete_instance(&self.state, &params.0.path).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // SCRIPT TOOLS
    // ═══════════════════════════════════════════

    #[tool(description = "Get the source code of a script with line numbers. Works with Script, LocalScript, and ModuleScript.")]
    async fn get_script_source(
        &self,
        params: Parameters<GetScriptSourceParams>,
    ) -> String {
        match tools::scripts::get_script_source(&self.state, &params.0.path).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Replace the entire source code of a script. Records a waypoint for undo support.")]
    async fn set_script_source(
        &self,
        params: Parameters<SetScriptSourceParams>,
    ) -> String {
        match tools::scripts::set_script_source(&self.state, &params.0.path, &params.0.source).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Search all scripts in the place for a text pattern. Returns matching lines with line numbers and file paths.")]
    async fn grep_scripts(
        &self,
        params: Parameters<GrepScriptsParams>,
    ) -> String {
        match tools::scripts::grep_scripts(&self.state, &params.0.pattern, params.0.case_sensitive).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Search for instances by name or class across the entire place. Use searchBy: 'name', 'class', or 'both'.")]
    async fn search_objects(
        &self,
        params: Parameters<SearchObjectsParams>,
    ) -> String {
        match tools::scripts::search_objects(&self.state, &params.0.query, params.0.search_by.as_deref()).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // UNDO / REDO
    // ═══════════════════════════════════════════

    #[tool(description = "Undo the last action in Roblox Studio using ChangeHistoryService.")]
    async fn undo(&self) -> String {
        match tools::history::undo(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Redo the last undone action in Roblox Studio using ChangeHistoryService.")]
    async fn redo(&self) -> String {
        match tools::history::redo(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // SESSION MANAGEMENT (Multi-Place Support)
    // ═══════════════════════════════════════════

    #[tool(description = "List all connected Roblox Studio sessions. Each open Studio instance registers as a separate session with its PlaceId and name.")]
    async fn list_sessions(&self) -> String {
        match tools::session::list_sessions(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Switch the active session to a different Studio instance. All subsequent tool calls will be routed to this session.")]
    async fn switch_session(
        &self,
        params: Parameters<SwitchSessionParams>,
    ) -> String {
        match tools::session::switch_session(&self.state, &params.0.session_id).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Get information about the currently active Studio session (PlaceId, name, connection status).")]
    async fn get_active_session(&self) -> String {
        match tools::session::get_active_session(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }
}

#[tool_handler]
impl ServerHandler for StudioLinkMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "StudioLink — Advanced Roblox Studio MCP Server with 49 tools for professional game development".into(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "StudioLink".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
