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
    /// (v0.6 EXPERIMENTAL) Optional session_id to route this single call to a
    /// specific Studio session, overriding active_session. Pass-through is
    /// observable at GET http://127.0.0.1:34872/debug/routing — used to verify
    /// whether the MCP client is shipping the field.
    pub session_id: Option<String>,
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
    /// Maximum number of pages to scan (default: 1)
    pub max_pages: Option<u32>,
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

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SetMySessionParams {
    /// Session ID to bind to this MCP instance. Pass null to clear and fall back to active_session.
    pub session_id: Option<String>,
}

// --- Place Publishing ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct PlaceVersionHistoryParams {
    /// PlaceId to query. Defaults to the active session's place_id.
    pub place_id: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct PublishPlaceParams {
    /// Version type: "Saved" (default) or "Published".
    pub version_type: Option<String>,
}

// --- Multi-Client Testing ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MultiClientTestParams {
    /// Number of clients to spawn (1-8). Default: 2.
    pub num_players: Option<u32>,
}

// --- Character Control (in-play) ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CharacterMovetoParams {
    /// Target position [x, y, z].
    pub target: [f64; 3],
    /// Player username or "@first" (default).
    pub player: Option<String>,
    /// If true (default), block until MoveToFinished or timeout.
    pub wait_finished: Option<bool>,
    /// Timeout in seconds when wait_finished=true. Default: 8.
    pub timeout_secs: Option<u32>,
    /// Route this call to a specific session_id (multi-chat / multi-place safe). Get ids from list_sessions. When multiple sessions exist ALWAYS pass this — relying on active_session is racy across chats.
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CharacterTeleportParams {
    /// [x,y,z] for position only, or [x,y,z, lookX,lookY,lookZ] for position + look-at.
    pub target: Vec<f64>,
    /// Player username or "@first" (default).
    pub player: Option<String>,
    /// Anchor HumanoidRootPart for one frame to avoid physics blowups. Default: false.
    pub anchor_during: Option<bool>,
    /// Route this call to a specific session_id (multi-chat / multi-place safe). Get ids from list_sessions. When multiple sessions exist ALWAYS pass this — relying on active_session is racy across chats.
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CharacterActionParams {
    /// One of: jump, sit, unsit, set_walkspeed, set_jumppower, set_health, heal, kill.
    pub action: String,
    /// Required for set_walkspeed / set_jumppower / set_health.
    pub value: Option<f64>,
    /// Player username or "@first" (default).
    pub player: Option<String>,
    /// Route this call to a specific session_id (multi-chat / multi-place safe). Get ids from list_sessions. When multiple sessions exist ALWAYS pass this — relying on active_session is racy across chats.
    pub session_id: Option<String>,
}

// --- Test Scenario Primitives ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct WaitForConditionParams {
    /// Dot-separated path to the instance, e.g. "Workspace.Counter".
    pub instance_path: String,
    /// Property name to read each poll, e.g. "Value".
    pub property: String,
    /// Comparison operator: ==, !=, >, >=, <, <=. Default: ==.
    pub operator: Option<String>,
    /// Value to compare against.
    pub target: Value,
    /// Poll interval in milliseconds. Default: 100.
    pub poll_interval_ms: Option<u32>,
    /// Timeout in seconds (max 110). Default: 30.
    pub timeout_secs: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct WaitForEventParams {
    /// Dot-separated path to the instance hosting the event.
    pub instance_path: String,
    /// Event property name, e.g. "Touched", "OnServerEvent", "Changed".
    pub event_name: String,
    /// Timeout in seconds (max 110). Default: 30.
    pub timeout_secs: Option<u32>,
    /// If true (default), captured args (stringified) are returned on fire.
    pub capture_args: Option<bool>,
}

// --- UI Manipulation (in-play) ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct UiClickParams {
    /// Selector: {"path": "PlayerGui.HUD.PlayBtn"} or {"tag": "..."} or {"attribute": {"key", "value"}}.
    pub selector: Value,
    /// Player username or "@first" (default).
    pub player: Option<String>,
    /// Route this call to a specific session_id (multi-chat / multi-place safe). Get ids from list_sessions. When multiple sessions exist ALWAYS pass this — relying on active_session is racy across chats.
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct UiSetTextParams {
    /// Selector for a TextBox / TextLabel / TextButton.
    pub selector: Value,
    /// New text value.
    pub text: String,
    /// Player username or "@first" (default).
    pub player: Option<String>,
    /// Route this call to a specific session_id (multi-chat / multi-place safe). Get ids from list_sessions. When multiple sessions exist ALWAYS pass this — relying on active_session is racy across chats.
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct UiGetStateParams {
    /// Selector for a GuiObject.
    pub selector: Value,
    /// Property names to read. Default: Text, Visible, AbsolutePosition, AbsoluteSize, Position, Size.
    pub properties: Option<Vec<String>>,
    /// Player username or "@first" (default).
    pub player: Option<String>,
    /// Route this call to a specific session_id (multi-chat / multi-place safe). Get ids from list_sessions. When multiple sessions exist ALWAYS pass this — relying on active_session is racy across chats.
    pub session_id: Option<String>,
}

// --- Input Simulation ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct InputSimulateParams {
    /// Array of action objects. Each action has a "type" field: "key" {key, mode?, duration_ms?}, "mouse_click" {position, button?, hold_ms?}, "mouse_move" {position}, or "key_combo" {keys, duration_ms?}.
    pub actions: Vec<Value>,
    /// Strategy: "vim" (direct VirtualInputManager) | "injection" (NOT YET IMPLEMENTED) | "auto" (default).
    pub strategy: Option<String>,
    /// Delay between actions in milliseconds. Default: 16 (~1 frame at 60fps).
    pub between_action_delay_ms: Option<u32>,
}

// --- Attributes & Tags (v0.8.0) ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AttrPathParams {
    /// Instance path, e.g. "Workspace.Model.Part".
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SetAttributeParams {
    /// Instance path.
    pub path: String,
    /// Attribute name.
    pub name: String,
    /// Attribute value (number/bool/string, or an object/array for Vector3/Color3/UDim2).
    pub value: serde_json::Value,
    /// Optional type hint: number, boolean, Vector3, Color3, UDim2, BrickColor, Enum.
    pub value_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DeleteAttributeParams {
    /// Instance path.
    pub path: String,
    /// Attribute name to remove.
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct BulkSetAttributesParams {
    /// Instance path.
    pub path: String,
    /// Map of attribute name -> value (value may be plain, or {value, type}).
    pub attributes: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct TagParams {
    /// Instance path.
    pub path: String,
    /// CollectionService tag name.
    pub tag: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetTaggedParams {
    /// CollectionService tag name.
    pub tag: String,
}

// --- Surgical script editing (v0.8.0) ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct EditScriptLinesParams {
    /// Script instance path.
    pub path: String,
    /// Exact text to replace (must match the source exactly).
    pub old_string: String,
    /// Replacement text.
    pub new_string: String,
    /// Optional 1-indexed line to begin the search from (disambiguates).
    pub start_line: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct InsertScriptLinesParams {
    /// Script instance path.
    pub path: String,
    /// Insert after this 1-indexed line (0 = before the first line).
    pub after_line: u32,
    /// Content to insert (may be multiple lines).
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DeleteScriptLinesParams {
    /// Script instance path.
    pub path: String,
    /// First line to delete (1-indexed, inclusive).
    pub start_line: u32,
    /// Last line to delete (1-indexed, inclusive).
    pub end_line: u32,
}

// --- Discovery (v0.8.0) ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct InspectInstanceParams {
    /// Instance path.
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetDescendantsParams {
    /// Instance path.
    pub path: String,
    /// Max recursion depth (default 10).
    pub max_depth: Option<u32>,
    /// Optional class filter (IsA — "BasePart" matches Part/MeshPart/…).
    pub class_filter: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SearchByPropertyParams {
    /// Property name to match.
    pub property_name: String,
    /// Value to match (compared as a string).
    pub property_value: Value,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetClassInfoParams {
    /// Roblox class name (e.g. "Part", "Humanoid").
    pub class_name: String,
}

// --- Find & Replace (v0.8.0) ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct FindReplaceParams {
    /// Text (or Lua pattern, if use_pattern) to find.
    pub pattern: String,
    /// Replacement text.
    pub replacement: String,
    /// Treat `pattern` as a Lua pattern (enables %d, captures %1, …). Default false.
    pub use_pattern: Option<bool>,
    /// Preview only — count matches without writing. Default false.
    pub dry_run: Option<bool>,
    /// Optional scope: only scripts whose full name contains this substring.
    pub path: Option<String>,
    /// Cap total replacements (default 1000).
    pub max_replacements: Option<u32>,
}

// --- Mass operations (v0.8.0) ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CloneObjectParams {
    /// Instance path to clone.
    pub path: String,
    /// Optional target parent path (default: same parent as the source).
    pub target_parent: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MassGetPropertyParams {
    /// Instance paths to read.
    pub paths: Vec<String>,
    /// Property name to read on each.
    pub property_name: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SmartDuplicateParams {
    /// Instance path to duplicate.
    pub path: String,
    /// Number of copies (>= 1).
    pub count: u32,
    /// Optional name pattern; "{n}" is replaced with the copy index.
    pub name_pattern: Option<String>,
    /// Optional cumulative position offset [x,y,z] applied per copy.
    pub position_offset: Option<Vec<f64>>,
}

// --- Logs / Errors ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ErrorHistoryParams {
    /// Filter by message type: "Output", "Info", "Warning", "Error". Omit for all.
    pub message_type: Option<String>,
    /// Substring filter (plain text, not regex).
    pub pattern: Option<String>,
    /// Max entries to return (newest first). Default: 100.
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CrashDumpParams {
    /// Time window in seconds. Default: 30.
    pub window_secs: Option<u32>,
}

// --- Script Patch ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ScriptPatchParams {
    /// Dot-separated path to the script (e.g. "ReplicatedStorage.Modules.Player").
    pub module_path: String,
    /// New source code to apply.
    pub new_source: String,
}

// --- Microprofiler ---

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MicroprofilerCaptureParams {
    /// Luau code to execute and profile.
    pub code: String,
    /// Optional label for debug.profilebegin (default: "studiolink_capture").
    pub label: Option<String>,
}

// ═══════════════════════════════════════════════════════
// MCP SERVER HANDLER
// ═══════════════════════════════════════════════════════

/// StudioLink MCP Server handler — registers and dispatches all 67 tools
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

    #[tool(
        description = "Execute Luau code in Roblox Studio and return the printed output. Can be used to both make changes and retrieve information."
    )]
    async fn run_code(&self, params: Parameters<RunCodeParams>) -> String {
        let p = params.0;
        match tools::core::run_code(&self.state, p.session_id.as_deref(), &p.command).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Search and insert a model from the Roblox Creator Store into the workspace."
    )]
    async fn insert_model(&self, params: Parameters<InsertModelParams>) -> String {
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

    #[tool(
        description = "Start or stop play mode or run the server. Mode must be 'start_play', 'stop', or 'run_server'."
    )]
    async fn start_stop_play(&self, params: Parameters<StartStopPlayParams>) -> String {
        match tools::core::start_stop_play(&self.state, &params.0.mode).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Run a Luau script in play mode with automatic stop after completion or timeout. Returns structured output with logs, errors, and duration."
    )]
    async fn run_script_in_play_mode(
        &self,
        params: Parameters<RunScriptInPlayModeParams>,
    ) -> String {
        match tools::core::run_script_in_play_mode(
            &self.state,
            &params.0.code,
            &params.0.mode,
            params.0.timeout,
        )
        .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Get the current Roblox Studio mode: 'start_play', 'run_server', or 'stop'."
    )]
    async fn get_studio_mode(&self) -> String {
        match tools::core::get_studio_mode(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // FAZ 2: DATASTORE & PROFILING
    // ═══════════════════════════════════════════

    #[tool(
        description = "List all DataStore names in the current experience. Requires 'Allow Studio Access to API Services' enabled in game settings."
    )]
    async fn datastore_list(&self) -> String {
        match tools::datastore::datastore_list(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Read a specific key's value from a DataStore.")]
    async fn datastore_get(&self, params: Parameters<DataStoreGetParams>) -> String {
        match tools::datastore::datastore_get(&self.state, &params.0.store_name, &params.0.key)
            .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Write a value to a DataStore key. WARNING: This modifies live production data."
    )]
    async fn datastore_set(&self, params: Parameters<DataStoreSetParams>) -> String {
        match tools::datastore::datastore_set(
            &self.state,
            &params.0.store_name,
            &params.0.key,
            params.0.value,
        )
        .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Delete a key from a DataStore. WARNING: This permanently removes live production data."
    )]
    async fn datastore_delete(&self, params: Parameters<DataStoreDeleteParams>) -> String {
        match tools::datastore::datastore_delete(&self.state, &params.0.store_name, &params.0.key)
            .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Scan and list all keys in a DataStore with pagination support.")]
    async fn datastore_scan(&self, params: Parameters<DataStoreScanParams>) -> String {
        match tools::datastore::datastore_scan(
            &self.state,
            &params.0.store_name,
            params.0.page_size,
            params.0.max_pages,
        )
        .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Start the ScriptProfiler to measure CPU time per function. Optional frequency in Hz (default: 1000)."
    )]
    async fn profile_start(&self, params: Parameters<ProfileStartParams>) -> String {
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

    #[tool(
        description = "Analyze profiling data: slowest functions, CPU hotspots, and optimization suggestions."
    )]
    async fn profile_analyze(&self) -> String {
        match tools::profiler::profile_analyze(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // FAZ 3: DIFFING & TESTING
    // ═══════════════════════════════════════════

    #[tool(
        description = "Take a snapshot of the current place state (all instances, properties, scripts). Optional name for the snapshot."
    )]
    async fn snapshot_take(&self, params: Parameters<SnapshotTakeParams>) -> String {
        match tools::diffing::snapshot_take(&self.state, params.0.name.as_deref()).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Compare two snapshots and list all differences (added/removed/changed instances and properties)."
    )]
    async fn snapshot_compare(&self, params: Parameters<SnapshotCompareParams>) -> String {
        match tools::diffing::snapshot_compare(
            &self.state,
            &params.0.snapshot_a,
            &params.0.snapshot_b,
        )
        .await
        {
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

    #[tool(
        description = "Run TestEZ test suites. Optionally specify a path to run tests for a specific module."
    )]
    async fn test_run(&self, params: Parameters<TestRunParams>) -> String {
        match tools::testing::test_run(&self.state, params.0.path.as_deref()).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Generate a TestEZ test template for a given script or ModuleScript.")]
    async fn test_create(&self, params: Parameters<TestCreateParams>) -> String {
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

    #[tool(
        description = "Scan the entire place for security vulnerabilities: unvalidated RemoteEvents, client trust issues, exposed data, missing rate limiting."
    )]
    async fn security_scan(&self) -> String {
        match tools::security::security_scan(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Get a formatted security report with risk levels (Critical/High/Medium/Low) and remediation suggestions."
    )]
    async fn security_report(&self) -> String {
        match tools::security::security_report(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Map all require() chains across the project. Detects circular dependencies, dead code (unrequired modules), and usage statistics."
    )]
    async fn dependency_map(&self) -> String {
        match tools::dependencies::dependency_map(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Scan for potential memory leaks: undisconnected Connections, undestroyed instances, growing tables, excessive RunService bindings."
    )]
    async fn memory_scan(&self) -> String {
        match tools::memory::memory_scan(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Analyze scripts for code quality: deprecated APIs, anti-patterns, naming issues, unused variables, missing type annotations."
    )]
    async fn lint_scripts(&self, params: Parameters<LintScriptsParams>) -> String {
        match tools::linter::lint_scripts(&self.state, params.0.path.as_deref()).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // FAZ 5: INSPECTOR TOOLS
    // ═══════════════════════════════════════════

    #[tool(
        description = "List all animations in the place with their IDs, durations, and priorities."
    )]
    async fn animation_list(&self) -> String {
        match tools::animation::animation_list(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Get detailed keyframe information for a specific animation.")]
    async fn animation_inspect(&self, params: Parameters<AnimationInspectParams>) -> String {
        match tools::animation::animation_inspect(&self.state, &params.0.animation_id).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Detect conflicting animations that affect the same body parts simultaneously."
    )]
    async fn animation_conflicts(&self) -> String {
        match tools::animation::animation_conflicts(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Start monitoring all RemoteEvent and RemoteFunction traffic (call frequency, data size, spam detection)."
    )]
    async fn network_monitor_start(&self) -> String {
        match tools::network::network_monitor_start(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Stop network monitoring and return a detailed traffic report with per-Remote statistics and bandwidth estimates."
    )]
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

    #[tool(
        description = "Detect UI issues: overlapping elements, off-screen UI, mobile touch target sizes, ZIndex conflicts, missing layout components."
    )]
    async fn ui_analyze(&self) -> String {
        match tools::ui_inspector::ui_analyze(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Auto-generate Markdown documentation for all ModuleScripts: public functions, parameter types, return types, dependencies."
    )]
    async fn docs_generate(&self, params: Parameters<DocsGenerateParams>) -> String {
        match tools::docs::docs_generate(&self.state, params.0.path.as_deref()).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // WORKSPACE ANALYSIS
    // ═══════════════════════════════════════════

    #[tool(
        description = "Comprehensive workspace analysis: coding style (naming, indent, strict mode, type annotations), architecture (framework, services, folder structure), script statistics, issues (deprecated APIs, security, memory leaks, optimization), dependencies (circular, dead modules), and detected patterns/libraries. Run this first on any new workspace."
    )]
    async fn workspace_analyze(&self, params: Parameters<WorkspaceAnalyzeParams>) -> String {
        match tools::workspace::workspace_analyze(&self.state, params.0.path.as_deref()).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // INSTANCE MANAGEMENT
    // ═══════════════════════════════════════════

    #[tool(
        description = "Get a hierarchical tree of all instances in the place. Optionally specify a path to focus on a subtree and depth to limit traversal."
    )]
    async fn get_file_tree(&self, params: Parameters<GetFileTreeParams>) -> String {
        match tools::instance::get_file_tree(&self.state, params.0.path.as_deref(), params.0.depth)
            .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Get all properties of an instance at the given path, including class-specific properties (BasePart, GuiObject, Light, etc.), attributes, and tags."
    )]
    async fn get_instance_properties(
        &self,
        params: Parameters<GetInstancePropertiesParams>,
    ) -> String {
        match tools::instance::get_instance_properties(&self.state, &params.0.path).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Set a single property on an instance. Supports type hints for Vector3, Color3, UDim2, BrickColor, Enum values."
    )]
    async fn set_property(&self, params: Parameters<SetPropertyParams>) -> String {
        match tools::instance::set_property(
            &self.state,
            &params.0.path,
            &params.0.property,
            params.0.value,
            params.0.value_type.as_deref(),
        )
        .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Set the same property on multiple instances at once. Provide an array of paths."
    )]
    async fn mass_set_property(&self, params: Parameters<MassSetPropertyParams>) -> String {
        match tools::instance::mass_set_property(
            &self.state,
            params.0.paths,
            &params.0.property,
            params.0.value,
            params.0.value_type.as_deref(),
        )
        .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Create a new instance with the given class name under a parent path. Optionally set initial properties."
    )]
    async fn create_instance(&self, params: Parameters<CreateInstanceParams>) -> String {
        match tools::instance::create_instance(
            &self.state,
            &params.0.class_name,
            params.0.parent_path.as_deref(),
            params.0.properties,
        )
        .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Delete an instance and all its descendants at the given path.")]
    async fn delete_instance(&self, params: Parameters<DeleteInstanceParams>) -> String {
        match tools::instance::delete_instance(&self.state, &params.0.path).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // SCRIPT TOOLS
    // ═══════════════════════════════════════════

    #[tool(
        description = "Get the source code of a script with line numbers. Works with Script, LocalScript, and ModuleScript."
    )]
    async fn get_script_source(&self, params: Parameters<GetScriptSourceParams>) -> String {
        match tools::scripts::get_script_source(&self.state, &params.0.path).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Replace the entire source code of a script. Records a waypoint for undo support."
    )]
    async fn set_script_source(&self, params: Parameters<SetScriptSourceParams>) -> String {
        match tools::scripts::set_script_source(&self.state, &params.0.path, &params.0.source).await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Search all scripts in the place for a text pattern. Returns matching lines with line numbers and file paths."
    )]
    async fn grep_scripts(&self, params: Parameters<GrepScriptsParams>) -> String {
        match tools::scripts::grep_scripts(&self.state, &params.0.pattern, params.0.case_sensitive)
            .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Search for instances by name or class across the entire place. Use searchBy: 'name', 'class', or 'both'."
    )]
    async fn search_objects(&self, params: Parameters<SearchObjectsParams>) -> String {
        match tools::scripts::search_objects(
            &self.state,
            &params.0.query,
            params.0.search_by.as_deref(),
        )
        .await
        {
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

    #[tool(
        description = "Redo the last undone action in Roblox Studio using ChangeHistoryService."
    )]
    async fn redo(&self) -> String {
        match tools::history::redo(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // SESSION MANAGEMENT (Multi-Place Support)
    // ═══════════════════════════════════════════

    #[tool(
        description = "List all connected Roblox Studio sessions. CALL THIS FIRST in every conversation that touches Studio. Each open Studio window is a separate session with its own session_id. If more than one session exists, pick the one this chat should drive and pass session_id on every subsequent tool call (run_code, character_*, ui_*, start_stop_play, etc.) — do not rely on active_session in multi-chat / multi-place setups."
    )]
    async fn list_sessions(&self) -> String {
        match tools::session::list_sessions(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Switch the active session to a different Studio instance. All subsequent tool calls will be routed to this session."
    )]
    async fn switch_session(&self, params: Parameters<SwitchSessionParams>) -> String {
        match tools::session::switch_session(&self.state, &params.0.session_id).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Get information about the currently active Studio session (PlaceId, name, connection status)."
    )]
    async fn get_active_session(&self) -> String {
        match tools::session::get_active_session(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Return the last 50 tool dispatches with their target_session value (multi-chat routing log). target_session=null means the call routed to active_session; a string means it was an explicit per-call session_id override. Mirrors GET http://127.0.0.1:34872/debug/routing."
    )]
    async fn debug_routing(&self) -> String {
        match tools::debug::debug_routing(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Bind this Claude/Cursor chat to a specific Studio session for the rest of the conversation. After calling set_my_session(session_id), every subsequent tool call WITHOUT an explicit session_id will automatically route to the bound session — no more passing session_id on every call. Pass null/none to clear and fall back to active_session. RECOMMENDED FLOW: list_sessions → ask user (or infer) which place this chat owns → set_my_session(<that_id>) once → forget about session_id for the rest."
    )]
    async fn set_my_session(&self, params: Parameters<SetMySessionParams>) -> String {
        match tools::affinity::set_my_session(&self.state, params.0.session_id).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Read the bound_session_id for this MCP instance (set via set_my_session) along with the global active_session. Returns null when nothing is bound."
    )]
    async fn get_my_session(&self) -> String {
        match tools::affinity::get_my_session(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // PLACE PUBLISHING
    // ═══════════════════════════════════════════

    #[tool(
        description = "List published versions of a place. Currently returns {supported: false} because Open Cloud does not yet expose a versions:list endpoint (5/2026). Use Studio's File > Game Settings > Versions for now."
    )]
    async fn place_version_history(&self, params: Parameters<PlaceVersionHistoryParams>) -> String {
        match tools::publish::place_version_history(&self.state, params.0.place_id).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Open Studio's publish dialog for the active place. version_type is 'Saved' (default) or 'Published'. The user must complete the dialog manually — true headless publish requires RobloxScriptSecurity which plugins don't have. Returns immediately with dialog_opened=true."
    )]
    async fn publish_place(&self, params: Parameters<PublishPlaceParams>) -> String {
        match tools::publish::publish_place(&self.state, params.0.version_type).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // MULTI-CLIENT TESTING
    // ═══════════════════════════════════════════

    #[tool(
        description = "Start a play-mode test with N clients (1-8, default 2). Wraps StudioTestService:ExecutePlayModeAsync. After it starts, each client + the server register as separate StudioLink sessions — use list_sessions to see them and switch_session to route tool calls. Returns immediately; play continues until stopped."
    )]
    async fn multi_client_test(&self, params: Parameters<MultiClientTestParams>) -> String {
        match tools::multi_client::multi_client_test(&self.state, params.0.num_players).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // ASSET AUDIT
    // ═══════════════════════════════════════════

    #[tool(
        description = "Inventory all meshes, textures, sounds, and animations across Workspace, ReplicatedStorage, ServerStorage, StarterGui, and StarterPlayer. Returns reuse count + example paths + total_seconds (audio/anim) per asset id. NOTE: per-asset byte size is not exposed by Roblox plugin APIs."
    )]
    async fn asset_audit(&self) -> String {
        match tools::asset_audit::asset_audit(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // INPUT (Faz 2 / v0.4.0)
    // ═══════════════════════════════════════════

    #[tool(
        description = "Probe VirtualInputManager methods (SendKeyEvent, SendMouseButtonEvent, etc.) to find out which are callable in the current Studio context. Run in Edit mode AND during Play to compare security levels. Result is cached on _G.StudioLink_VimReport for input_simulate's strategy selection."
    )]
    async fn vim_capability_test(&self) -> String {
        match tools::input::vim_capability_test(&self.state).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // CHARACTER CONTROL (Faz 2 / v0.4.0)
    // ═══════════════════════════════════════════

    #[tool(
        description = "Walk a player's character to (x,y,z) via Humanoid:MoveTo. Default waits for MoveToFinished (8s timeout); set wait_finished=false for fire-and-forget. Requires play mode. Switch to the play-server session first via switch_session."
    )]
    async fn character_moveto(&self, params: Parameters<CharacterMovetoParams>) -> String {
        let p = params.0;
        match tools::character::character_moveto(
            &self.state,
            p.session_id.as_deref(),
            p.target,
            p.player,
            p.wait_finished,
            p.timeout_secs,
        )
        .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Instant teleport via Player.Character:PivotTo. Pass [x,y,z] for position only or [x,y,z, lookX,lookY,lookZ] for position + look-at. anchor_during=true freezes the rootpart for 1 frame to avoid physics blowups."
    )]
    async fn character_teleport(&self, params: Parameters<CharacterTeleportParams>) -> String {
        let p = params.0;
        match tools::character::character_teleport(
            &self.state,
            p.session_id.as_deref(),
            p.target,
            p.player,
            p.anchor_during,
        )
        .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Combined Humanoid actions: jump | sit | unsit | set_walkspeed | set_jumppower | set_health | heal | kill. set_* and set_health require numeric value. Returns current_health afterwards."
    )]
    async fn character_action(&self, params: Parameters<CharacterActionParams>) -> String {
        let p = params.0;
        match tools::character::character_action(
            &self.state,
            p.session_id.as_deref(),
            p.action,
            p.value,
            p.player,
        )
        .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // TEST SCENARIO PRIMITIVES (Faz 2 / v0.4.0)
    // ═══════════════════════════════════════════

    #[tool(
        description = "Poll a property of an instance until it satisfies a comparison (==, !=, >, >=, <, <=) against target, or until timeout (max 110s). Returns satisfied=true on match, satisfied=false on timeout."
    )]
    async fn wait_for_condition(&self, params: Parameters<WaitForConditionParams>) -> String {
        let p = params.0;
        match tools::scenario::wait_for_condition(
            &self.state,
            p.instance_path,
            p.property,
            p.operator,
            p.target,
            p.poll_interval_ms,
            p.timeout_secs,
        )
        .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Connect to an event property of an instance (Touched, OnServerEvent, Changed, etc.) and wait for it to fire once or until timeout (max 110s). Returns fired=true with captured_args (stringified) on success."
    )]
    async fn wait_for_event(&self, params: Parameters<WaitForEventParams>) -> String {
        let p = params.0;
        match tools::scenario::wait_for_event(
            &self.state,
            p.instance_path,
            p.event_name,
            p.timeout_secs,
            p.capture_args,
        )
        .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // UI MANIPULATION (Faz 2 / v0.4.0)
    // ═══════════════════════════════════════════

    #[tool(
        description = "Trigger a GuiButton's Activated event via gui:Activate(). Selector accepts {path: 'PlayerGui.HUD.PlayBtn'} (path under player), {tag: '...'}, or {attribute: {key, value}}. Server-side listeners fire immediately; client-side listeners run via replication."
    )]
    async fn ui_click(&self, params: Parameters<UiClickParams>) -> String {
        let p = params.0;
        match tools::ui::ui_click(&self.state, p.session_id.as_deref(), p.selector, p.player).await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Set the Text property of a TextBox / TextLabel / TextButton. Server-side property write replicates to the client. Returns previous_text and new_text."
    )]
    async fn ui_set_text(&self, params: Parameters<UiSetTextParams>) -> String {
        let p = params.0;
        match tools::ui::ui_set_text(
            &self.state,
            p.session_id.as_deref(),
            p.selector,
            p.text,
            p.player,
        )
        .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Read selected properties of a GuiObject. Default: Text, Visible, AbsolutePosition, AbsoluteSize, Position, Size. Vector2/UDim2/Color3 values serialize to arrays."
    )]
    async fn ui_get_state(&self, params: Parameters<UiGetStateParams>) -> String {
        let p = params.0;
        match tools::ui::ui_get_state(
            &self.state,
            p.session_id.as_deref(),
            p.selector,
            p.properties,
            p.player,
        )
        .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // INPUT SIMULATION (Faz 2 / v0.4.0)
    // ═══════════════════════════════════════════

    #[tool(
        description = "Drive Studio's keyboard/mouse via VirtualInputManager. Each action is {type, ...}: 'key' {key, mode='tap'|'press'|'release', duration_ms}, 'mouse_click' {position:[x,y], button='Left'|'Right'|'Middle', hold_ms}, 'mouse_move' {position:[x,y]}, 'key_combo' {keys:[...]}. Strategy 'vim' (direct) or 'auto' (default). Run vim_capability_test first."
    )]
    async fn input_simulate(&self, params: Parameters<InputSimulateParams>) -> String {
        let p = params.0;
        match tools::input::input_simulate(
            &self.state,
            p.actions,
            p.strategy,
            p.between_action_delay_ms,
        )
        .await
        {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // VIEWPORT CAPTURE (v0.8.0)
    // ═══════════════════════════════════════════

    #[tool(
        description = "Capture the current Studio viewport as a PNG image (returned as an image you can see). Uses CaptureService + EditableImage in-engine — NO macOS Screen Recording / OS permission needed. Requires Edit mode with the viewport visible and Game Settings > Security > 'Allow Mesh / Image APIs' enabled. Returns at native viewport resolution."
    )]
    async fn viewport_capture(&self) -> Content {
        match tools::screenshot::viewport_capture(&self.state).await {
            Ok((png, _w, _h)) => {
                use base64::Engine;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
                Content::image(b64, "image/png")
            }
            Err(e) => Content::text(format!("Error: {}", e)),
        }
    }

    // ═══════════════════════════════════════════
    // ATTRIBUTES & TAGS (v0.8.0)
    // ═══════════════════════════════════════════

    #[tool(description = "Get all custom attributes on an instance (name -> {value, type}).")]
    async fn get_attributes(&self, params: Parameters<AttrPathParams>) -> String {
        match tools::attributes::get_attributes(&self.state, &params.0.path).await {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Set one custom attribute on an instance. value_type coerces typed values (Vector3/Color3/UDim2/BrickColor/Enum)."
    )]
    async fn set_attribute(&self, params: Parameters<SetAttributeParams>) -> String {
        let p = params.0;
        match tools::attributes::set_attribute(
            &self.state,
            &p.path,
            &p.name,
            p.value,
            p.value_type.as_deref(),
        )
        .await
        {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Delete a custom attribute from an instance by name.")]
    async fn delete_attribute(&self, params: Parameters<DeleteAttributeParams>) -> String {
        let p = params.0;
        match tools::attributes::delete_attribute(&self.state, &p.path, &p.name).await {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Set many attributes on one instance in a single call. `attributes` is a map of name -> value (value may be plain, or {value, type})."
    )]
    async fn bulk_set_attributes(&self, params: Parameters<BulkSetAttributesParams>) -> String {
        let p = params.0;
        match tools::attributes::bulk_set_attributes(&self.state, &p.path, p.attributes).await {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Get the CollectionService tags on an instance.")]
    async fn get_tags(&self, params: Parameters<AttrPathParams>) -> String {
        match tools::attributes::get_tags(&self.state, &params.0.path).await {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Add a CollectionService tag to an instance.")]
    async fn add_tag(&self, params: Parameters<TagParams>) -> String {
        let p = params.0;
        match tools::attributes::add_tag(&self.state, &p.path, &p.tag).await {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Remove a CollectionService tag from an instance.")]
    async fn remove_tag(&self, params: Parameters<TagParams>) -> String {
        let p = params.0;
        match tools::attributes::remove_tag(&self.state, &p.path, &p.tag).await {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Find all instances carrying a CollectionService tag (returns full instance paths)."
    )]
    async fn get_tagged(&self, params: Parameters<GetTaggedParams>) -> String {
        match tools::attributes::get_tagged(&self.state, &params.0.tag).await {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // SURGICAL SCRIPT EDITING (v0.8.0)
    // ═══════════════════════════════════════════

    #[tool(
        description = "Replace an exact text span in a live Studio script's source (like an Edit tool — old_string must match exactly). Optional start_line anchors the search when old_string is ambiguous. Far cheaper than rewriting the whole script via set_script_source."
    )]
    async fn edit_script_lines(&self, params: Parameters<EditScriptLinesParams>) -> String {
        let p = params.0;
        match tools::scripts::edit_script_lines(
            &self.state,
            &p.path,
            &p.old_string,
            &p.new_string,
            p.start_line,
        )
        .await
        {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Insert content into a script after a given line (after_line=0 inserts before the first line). 1-indexed."
    )]
    async fn insert_script_lines(&self, params: Parameters<InsertScriptLinesParams>) -> String {
        let p = params.0;
        match tools::scripts::insert_script_lines(&self.state, &p.path, p.after_line, &p.content)
            .await
        {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Delete lines start_line..end_line (1-indexed, inclusive) from a script's source."
    )]
    async fn delete_script_lines(&self, params: Parameters<DeleteScriptLinesParams>) -> String {
        let p = params.0;
        match tools::scripts::delete_script_lines(&self.state, &p.path, p.start_line, p.end_line)
            .await
        {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // DISCOVERY (v0.8.0)
    // ═══════════════════════════════════════════

    #[tool(
        description = "Rich summary of one instance: class, custom attributes, CollectionService tags, and a compact child list. One call instead of several."
    )]
    async fn inspect_instance(&self, params: Parameters<InspectInstanceParams>) -> String {
        match tools::discovery::inspect_instance(&self.state, &params.0.path).await {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Recursive descendants of an instance, optionally filtered by class (IsA — 'BasePart' matches Part/MeshPart). Cheaper than repeated get_instance_children calls."
    )]
    async fn get_descendants(&self, params: Parameters<GetDescendantsParams>) -> String {
        let p = params.0;
        match tools::discovery::get_descendants(
            &self.state,
            &p.path,
            p.max_depth,
            p.class_filter.as_deref(),
        )
        .await
        {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "The instances currently selected in Studio.")]
    async fn get_selection(&self) -> String {
        match tools::discovery::get_selection(&self.state).await {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Find instances whose named property equals a value (compared as a string). Scans the main services."
    )]
    async fn search_by_property(&self, params: Parameters<SearchByPropertyParams>) -> String {
        let p = params.0;
        match tools::discovery::search_by_property(&self.state, &p.property_name, p.property_value)
            .await
        {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "The main game services and their child counts.")]
    async fn get_services(&self) -> String {
        match tools::discovery::get_services(&self.state).await {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(description = "Place id, game id, name, creator, and place version.")]
    async fn get_place_info(&self) -> String {
        match tools::discovery::get_place_info(&self.state).await {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Validate a class name and report its IsA hierarchy (e.g. Part -> BasePart -> PVInstance -> Instance)."
    )]
    async fn get_class_info(&self, params: Parameters<GetClassInfoParams>) -> String {
        match tools::discovery::get_class_info(&self.state, &params.0.class_name).await {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Project-wide find/replace across all script sources. Literal by default; use_pattern enables Lua patterns (captures via %1). dry_run previews without writing. path scopes by full-name substring. max_replacements caps it (default 1000)."
    )]
    async fn find_and_replace_in_scripts(&self, params: Parameters<FindReplaceParams>) -> String {
        let p = params.0;
        match tools::scripts::find_and_replace_in_scripts(
            &self.state,
            &p.pattern,
            &p.replacement,
            p.use_pattern,
            p.dry_run,
            p.path.as_deref(),
            p.max_replacements,
        )
        .await
        {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // MASS OPERATIONS (v0.8.0)
    // ═══════════════════════════════════════════

    #[tool(
        description = "Deep-copy an instance (and its descendants) under a parent (default: the source's own parent)."
    )]
    async fn clone_object(&self, params: Parameters<CloneObjectParams>) -> String {
        let p = params.0;
        match tools::mass::clone_object(&self.state, &p.path, p.target_parent.as_deref()).await {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Read one property across many instances in a single call. Returns a path -> value map."
    )]
    async fn mass_get_property(&self, params: Parameters<MassGetPropertyParams>) -> String {
        let p = params.0;
        match tools::mass::mass_get_property(&self.state, p.paths, &p.property_name).await {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Duplicate an instance `count` times with an optional name pattern ('{n}' -> copy index) and a cumulative position offset [x,y,z] per copy."
    )]
    async fn smart_duplicate(&self, params: Parameters<SmartDuplicateParams>) -> String {
        let p = params.0;
        match tools::mass::smart_duplicate(
            &self.state,
            &p.path,
            p.count,
            p.name_pattern.as_deref(),
            p.position_offset,
        )
        .await
        {
            Ok(r) => ok_text(r),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // LOGS & ERRORS (Faz 3 / v0.5.0)
    // ═══════════════════════════════════════════

    #[tool(
        description = "Read LogService:GetLogHistory() entries with optional filtering by message_type (Output/Info/Warning/Error) and substring pattern. Returns up to `limit` newest matches (default 100)."
    )]
    async fn error_history(&self, params: Parameters<ErrorHistoryParams>) -> String {
        let p = params.0;
        match tools::logs::error_history(&self.state, p.message_type, p.pattern, p.limit).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    #[tool(
        description = "Snapshot recent log activity within window_secs (default 30) with the error subset isolated and stack-trace patterns flagged. Studio process crashes (.dmp) are NOT accessible from plugin context — this covers logical errors only."
    )]
    async fn crash_dump(&self, params: Parameters<CrashDumpParams>) -> String {
        match tools::logs::crash_dump(&self.state, params.0.window_secs).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // SCRIPT PATCH (Faz 3 / v0.5.0)
    // ═══════════════════════════════════════════

    #[tool(
        description = "Replace a Script/LocalScript/ModuleScript's source with diff stats and ChangeHistoryService waypoints. NOT live hot-reload: existing required ModuleScripts continue using the old version until next require() / play restart. Optional loadstring syntax check runs only if Studio has it enabled."
    )]
    async fn script_patch(&self, params: Parameters<ScriptPatchParams>) -> String {
        let p = params.0;
        match tools::script_patch::script_patch(&self.state, p.module_path, p.new_source).await {
            Ok(result) => ok_text(result),
            Err(e) => err_text(e),
        }
    }

    // ═══════════════════════════════════════════
    // MICROPROFILER (Faz 3 / v0.5.0)
    // ═══════════════════════════════════════════

    #[tool(
        description = "Wrap a Luau code block in debug.profilebegin/end and measure wall time + Lua heap delta. NOTE: Studio's MicroProfiler GUI export is not exposed by Roblox APIs — this is script-level profiling only (no per-frame Render/Physics/Network breakdown)."
    )]
    async fn microprofiler_capture(
        &self,
        params: Parameters<MicroprofilerCaptureParams>,
    ) -> String {
        let p = params.0;
        match tools::profiler_v2::microprofiler_capture(&self.state, p.code, p.label).await {
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
                "StudioLink — Advanced Roblox Studio MCP Server with 67 tools for professional Roblox game development.\n\
                \n\
                ═══════════════════════════════════════════════════════════════════\n\
                MULTI-SESSION / MULTI-CHAT WORKFLOW (READ THIS FIRST)\n\
                ═══════════════════════════════════════════════════════════════════\n\
                \n\
                A user may have several Roblox Studio windows open at once — each\n\
                registers as a separate session. Several Claude/Cursor chats may\n\
                also be talking to this same StudioLink server simultaneously.\n\
                \n\
                RECOMMENDED FLOW (uses session affinity, no per-call boilerplate):\n\
                  1. Call list_sessions FIRST to see how many sessions exist.\n\
                  2. If count == 1, you can skip the rest — that one is yours.\n\
                  3. If count > 1, ASK THE USER which place this chat owns\n\
                     (or infer from context: 'main game', 'test place', etc.).\n\
                  4. Call set_my_session(session_id=<chosen>) ONCE. This binds\n\
                     this MCP instance to that session for the rest of the\n\
                     conversation. The server remembers it; you don't need to\n\
                     pass session_id on subsequent tool calls.\n\
                  5. Just use tools normally afterward — every call routes to\n\
                     the bound session automatically.\n\
                \n\
                Routing precedence on every tool call:\n\
                    explicit session_id param > bound_session_id > active_session\n\
                \n\
                You can still pass session_id explicitly when you need to\n\
                temporarily target a different place for one call without\n\
                changing the binding.\n\
                \n\
                Verify what is bound: call get_my_session.\n\
                Verify what landed where: call debug_routing (last 50 calls).\n\
                Unbind: call set_my_session(session_id=null) or set_my_session()\n\
                with no argument to fall back to active_session.\n\
                \n\
                Each Claude/Cursor chat has its own studiolink process with its\n\
                own bound_session_id, so chats do not clobber each other.\n\
                ═══════════════════════════════════════════════════════════════════"
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "StudioLink".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
