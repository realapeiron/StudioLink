# Changelog

All notable changes to StudioLink. Format roughly follows [Keep a Changelog](https://keepachangelog.com/) and [SemVer](https://semver.org/).

## [v0.7.1] — Bug fixes & housekeeping

### Fixed
- **Proxy timeout**: `/proxy/tool_call` previously timed out at 60s, but tool functions can request `EXTENDED_TIMEOUT` (120s) for long-running operations like `asset_audit`, `wait_for_*`, and `multi_client_test`. Long calls were silently failing with `GATEWAY_TIMEOUT` from secondary chats. Bumped to 125s (120 + 5s buffer).
- **Server-context `run_code` killing `task.spawn` coroutines**: the dynamic Script runner used to call `runnerScript:Destroy()` immediately after collecting the result, which terminates any threads the user code spawned. Fire-and-forget patterns (`task.spawn(function() task.wait(2); … end)`) silently lost their continuations. Now the destroy is deferred 60s via `task.delay`, so spawned threads outlive the response while next-call cleanup still prevents leaks.

### Added
- `CHANGELOG.md` for human-readable release history.

## [v0.7.0] — Server-side session affinity

### Added
- `set_my_session(session_id)` — bind this MCP instance to a Studio session for the rest of the conversation. Subsequent tool calls without an explicit `session_id` automatically route to the bound session.
- `get_my_session()` — inspect bound + active session.
- Routing precedence: explicit `session_id` param > `bound_session_id` > `active_session`.
- AppState gains `bound_session_id`. Per-process isolation: each Claude/Cursor chat spawns its own studiolink instance, so each chat has its own bound session and they don't clobber each other.
- MCP `instructions` field rewritten around the new flow: `list_sessions → set_my_session(once) → just-use-tools`.

### Tools at this version: 67.

## [v0.6.0] — Multi-chat parallel editing

### Added
- `session_id` parameter on 7 in-game tools: `run_code`, `character_moveto`, `character_teleport`, `character_action`, `ui_click`, `ui_set_text`, `ui_get_state`.
- `PluginRequest.target_session` field: proxy mode now forwards routing context across the secondary→primary hop.
- `handle_proxy_tool_call` resolves explicit `target_session` before queueing.
- `debug_routing` tool + `GET /debug/routing` HTTP endpoint: ring buffer of the last 50 dispatches with their `target_session` value.

### Fixed
- Multi-chat session_id silent bypass: previous task_local approach didn't survive rmcp's tool dispatch boundary; switched to explicit parameter passing through `send_to_plugin` and added the missing 404 handling in `send_via_proxy`.

### Tools at this version: 65.

## [v0.5.0] — Debugging deep dive

### Added
- `error_history` — LogService:GetLogHistory() with message_type and pattern filters.
- `crash_dump` — windowed log snapshot with stack-trace flagging.
- `script_patch` — Script/LocalScript/ModuleScript source replace with diff stats and ChangeHistoryService waypoints.
- `microprofiler_capture` — `debug.profilebegin/end` wrapper measuring wall time + Lua heap delta.

### Tools at this version: 64.

## [v0.4.0] — In-game automation

### Added
- Foundation `PlayHelpers` util (player resolution, selector resolution, context guards).
- `vim_capability_test` — VirtualInputManager probe diagnostic.
- Character control: `character_moveto`, `character_teleport`, `character_action`.
- UI manipulation: `ui_click`, `ui_set_text`, `ui_get_state`.
- Test scenarios: `wait_for_condition`, `wait_for_event`.
- `input_simulate` — VirtualInputManager-driven keyboard/mouse with `vim`/`auto`/`injection` strategies (injection deferred to v0.8+).
- `viewport_screenshot` — macOS OS-level screencapture (limited under Claude Desktop sandbox).

### Tools at this version: 60.

## [v0.3.0] — Workflow closure

### Added
- `place_version_history` (stub — Open Cloud `versions:list` endpoint not yet documented).
- `multi_client_test` — wraps `StudioTestService:ExecutePlayModeAsync` for N-client play tests.
- `publish_place` — opens Studio's publish dialog (true headless publish requires RobloxScriptSecurity).
- `asset_audit` — inventory of meshes / textures / sounds / animations with reuse counts.

### Infrastructure
- CI now runs `cargo test --release` on every PR.
- 30 unit tests (was 0).

### Tools at this version: 53.

## [Pre-v0.3.0] — Historical fixes (commit history only)

- `5781b50` — Fix unpublished places (place_id=0) re-register loop. Skip dedup when place_id is 0; rely on heartbeat timeout for stale cleanup.
- `7b1d66b` — Cargo.lock sync to v0.2.0.
