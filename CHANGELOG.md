# Changelog

All notable changes to StudioLink. Format roughly follows [Keep a Changelog](https://keepachangelog.com/) and [SemVer](https://semver.org/).

## [v0.8.0] — Viewport capture + attributes/tags + surgical script editing

Feature update from a gap analysis against `boshyxd/robloxstudio-mcp` and the
official Roblox Studio MCP. 12 new tools (66 → 79).

### Added — Viewport capture (1 tool)
- **`viewport_capture`** — capture the Studio viewport as a PNG, returned as an
  MCP **image** (the model actually sees it). Fully in-engine:
  `CaptureService:CaptureScreenshot` → `EditableImage` → tiled `ReadPixelsBuffer`
  → base64 RGBA → server-side Rust PNG encode. **No OS Screen Recording
  permission** (unlike the `screencapture` path removed in v0.7.5, which was dead
  under the Claude Desktop sandbox). Edit mode, native resolution; requires Game
  Settings > Security > "Allow Mesh / Image APIs". The plugin posts multi-MB RGBA,
  so the server body limit was raised to 64 MB (no chunking needed).

### Added — Attributes & Tags (8 tools)
- `get_attributes`, `set_attribute`, `delete_attribute`, `bulk_set_attributes` —
  typed values reuse the existing `deserializeValue` coercion (Vector3/Color3/…).
- `get_tags`, `add_tag`, `remove_tag`, `get_tagged` — CollectionService.

### Added — Surgical script editing (3 tools)
- `edit_script_lines` — exact-text replace with an optional `start_line` anchor
  (an Edit-tool for live scripts; far cheaper than rewriting via
  `set_script_source`).
- `insert_script_lines` — insert after a line (`after_line=0` = before line 1).
- `delete_script_lines` — delete `start_line..end_line` (1-indexed inclusive).
- Pure line logic extracted to `Utils/LineEdit.luau`, unit-tested.

### Testing / deps
- New tests: Rust `rgba_to_png` + `attributes` + script-line validation (62 Rust
  tests); Lune `Base64` (RFC 4648 vectors) + `LineEdit`.
- Added `base64` + `png` crates (PNG encoded server-side; plugin ships raw RGBA,
  zero plugin-side compression).

### Added — Discovery (7 tools)
- `inspect_instance` (class + attributes + tags + child summary in one call),
  `get_descendants` (recursive, optional IsA class filter), `get_selection`,
  `search_by_property`, `get_services`, `get_place_info`, `get_class_info`.

### Added — Refactor & mass ops (4 tools)
- `find_and_replace_in_scripts` — project-wide, literal or Lua-pattern, `dry_run`
  preview, `path` scoping, `max_replacements` cap. Core logic in
  `Utils/FindReplace.luau`, unit-tested.
- `clone_object`, `mass_get_property`, `smart_duplicate` (per-copy name pattern
  `{n}` + cumulative position offset).

### Tools: 90 (was 66).

## [v0.7.5] — Remove the screenshot feature

Removed `viewport_screenshot` entirely. The macOS `screencapture` path needs
Screen Recording permission that can't be granted under the Claude Desktop
sandbox (parent bundle lacks the entitlement). The in-engine alternative
(`CaptureService:CaptureScreenshot` → `EditableImage:ReadPixelsBuffer`) works
without permission but is capped at 1024×1024 and can't fit a frame in one
~1 MB plugin POST without a chunked-transport protocol — not worth the
complexity here. Cut cleanly rather than ship something half-working.

### Removed
- `viewport_screenshot` tool — Rust `tools::screenshot` (`screenshot.rs`),
  plugin `ViewportScreenshot.luau`, the MCP tool definition + params struct in
  `mcp.rs`, and the plugin register entry.
- `base64` dependency (only the screenshot path used it).

### Tools: 66 (was 67). 51 Rust tests; clippy + fmt clean; plugin parses clean.

## [v0.7.4] — Multi-chat state isolation + audit tail

Closes the v0.7.x audit backlog.

### Fixed (plugin — multi-chat isolation)
- **Stateful tools clobbered each other across chats**: `profile_start`/`stop`,
  `test_run`/`report`, `snapshot_take`/`compare`/`list`, and `security_scan`/
  `report` held results in module-level locals, so two chats driving the *same*
  Studio session overwrote each other's in-flight state. Each studiolink process
  now stamps a unique `instance_id` into every request as `__caller_id` (carried
  in the args, so it survives both the direct queue and the proxy hop), and those
  four tool groups key their state by it. `security_scan`/`report` gained an
  `args` parameter to receive it. NetworkMonitor is single-active and unchanged.

### Fixed (LOW)
- **`viewport_screenshot` temp leak**: leftover `studiolink_capture_*.png` files
  from failed cleanups are reaped (older than 1h) at capture time. Reaper is
  unit-tested with a parameterized cutoff.
- **`memory_scan` false negatives**: cleanup-library detection (Maid/Janitor/
  Trove) now requires a `.`/`:` after the name, so a bare mention in a comment
  doesn't count as cleanup and mask a real leak.

### Still deferred (low value / behavior risk)
- `input_simulate` `PlayHelpers.requireContext` (would reject Edit-mode use);
  `ScriptPatch` loadstring syntax check (already fails safe with a warning).

### Tests at this version: 53 Rust + 3 Lune suites + parse check; clippy + fmt clean.

## [v0.7.3] — Second audit pass: plugin logic bugs + input validation

Follow-up audit (Rust + Luau split across parallel reviewers, then each finding
verified by hand). The genuine bugs below were fixed test-first where the logic
was pure: a Lune-based unit harness now covers the comparison/security
heuristics that have no Roblox dependency, plus a parse check over every plugin
source file.

### Fixed (plugin — Luau)
- **`wait_for_condition` equality didn't coerce types**: v0.7.2 added numeric
  coercion for ordering operators (`>`, `<`, …) but left `==` / `!=` doing raw
  comparison — so the *most common* operator still silently failed when a
  property read returned a number and the JSON arg was a numeric string (or vice
  versa): `100 == "100"` → false → wait times out on a condition that looks
  satisfied. All operators now coerce when both sides are numeric, while
  non-numeric strings still compare literally. Logic extracted to
  `Utils/NumCompare.luau` and unit-tested.
- **`security_scan` produced false negatives** (an unvalidated remote reported
  as safe): remote names were matched with `string.find` in *pattern* mode, so a
  name like `Data.Save` matched the unrelated `DataXSave` (`.` = any char) and a
  name like `Buy-Item` failed to match its own script (`-` = quantifier).
  Matching is now plain-text. Separately, a bare `if … then` anywhere in a script
  counted as "input validation" — almost every script has one — so the check was
  near-meaningless; only real `typeof` / `type` / `assert` guards count now.
  Validation scope also no longer includes client containers (StarterGui/Pack/
  Player). Heuristics extracted to `Utils/RemoteValidation.luau` and unit-tested.
- **`character_teleport` could freeze a character permanently**: in
  `anchor_during` mode, if `PivotTo` threw (e.g. a degenerate `CFrame.lookAt`)
  the `Anchored = false` restore never ran. Now wrapped so anchoring is always
  restored.
- **`start_stop_play` could strand the mode flag**: `ExecutePlayModeAsync` /
  `ExecuteRunModeAsync` ran un-`pcall`'d inside `task.defer`; a throw killed the
  thread and left `mode` stuck at `start_play`/`run_server` for the session.
  Now guarded.
- **`network_monitor` could throw on the hot path and leaked memory**:
  `JSONEncode` of remote args runs inside `OnServerEvent`, but `Instance` /
  `Vector3` / `CFrame` args (common) aren't encodable and threw; now guarded with
  a per-arg size fallback. A write-only `timestamps` array that grew on every
  fire and was never read back was removed.
- **`input_simulate` always reported success**: even when every action failed it
  returned `success = true`. Now reports failure when nothing executed (partial
  success stays truthy so the per-action breakdown survives the dispatch layer,
  which drops `result` on failure).
- **`run_script_in_play_mode` detected play via a stale `_G` flag**: code started
  via Studio's own Play button left the flag at `stop`, so the tool ran in the
  Edit DataModel. Now uses `RunService:IsRunning()`.

### Fixed (server — Rust)
- **Empty-input validation gaps**: `datastore_*`, `get`/`set_script_source`,
  `grep_scripts`, `search_objects`, and the `instance` tools accepted empty
  `store_name` / `key` / `path` / `pattern` / `class_name`, unlike their siblings
  (`script_patch` etc.). The plugin didn't backstop this — Lua treats `""` as
  truthy, so an empty key reached `GetDataStore("")`. All now reject empty input
  with a clear `InvalidArguments` error.

### Fixed (MEDIUM follow-up)
- **`create_instance` silently dropped typed properties**: `properties` were
  assigned raw with no `deserializeValue`, so `{Size={1,2,3}}` / `{Anchored="true"}`
  no-op'd while still reporting `created=true`. Each property's type is now
  inferred from the new instance and coerced like `set_property`.
- **`mass_set_property` threw on a non-table `paths`**: only a nil-check guarded
  the `ipairs`. Now type-checked.
- **`test_run` matched any name containing "test"**: pulled in `LatestConfig`,
  `ContestManager`, `TestUtilities` (running their top-level side effects) and ran
  any function with "test" anywhere in the name. Now `.spec`/`.test` suffix and a
  `test*` prefix only. Extracted to `Utils/TestMatch.luau` + unit-tested.
- **`snapshot_take` collided and leaked**: default names used `os.time()` (second
  resolution → same-second overwrite); now counter-suffixed. Snapshots (full
  serialized trees) are capped at 10 with oldest-eviction.
- **`profile_start` was a no-op**: fetched `Stats` and discarded it. Now captures
  a start `InstanceCount` so `profile_stop` reports a real interval delta, and the
  response is honest that it's engine Stats, not a per-script CPU profile (use
  `microprofiler_capture` for that).
- **`datastore_scan` paging unbounded** (Rust): `page_size`/`max_pages` now clamp
  to 1–100 / 1–20.
- **DataStore tools ignored request budget**: `datastore_get`/`set`/`delete` now
  check `GetRequestBudgetForRequestType` and return a clear throttle message
  instead of failing deep inside the call.

### Added (testing)
- Lune unit-test harness: `plugin/tests/{NumCompare,RemoteValidation,TestMatch}.test.luau`
  + `parse_check.luau` (syntax check over all 52 plugin source files). Run with
  `lune run plugin/tests/<file>`.
- 17 Rust validation/logic tests (`datastore`, `scripts`, `instance`).

### Known still-open: full multi-chat session-keying of module-level tool state
(Profiler/TestRunner/PlaceDiff/SecurityAudit) is architectural and deferred; some
LOW items (screenshot temp cleanup, MemoryLeakScan heuristic scoping) remain.

### Tests at this version: 50 Rust + 3 Lune suites + parse check; clippy + fmt clean.

## [v0.7.2] — Audit-driven cleanup

Three-agent code audit run; the genuine findings were addressed here.
Most agent flags were false positives (intentional design — auto-recovery
asymmetry, heartbeat 120s vs 45s gap, UUID collision, etc.) and dropped.

### Fixed
- **wait_for_condition mixed-type compare**: ordering operators (`>`, `>=`,
  `<`, `<=`) silently returned false when one side was a numeric string —
  e.g. `compare("100", ">", 50)`. Roblox property reads (StringValue.Value,
  attribute strings) frequently surface numeric strings, so the silent
  fail led to wait_for_condition timing out for what looked like a valid
  match. Now both sides go through `tonumber` coercion before comparison.
- **response_channels orphan leak**: when a tool timed out, its receiver
  was dropped but the matching sender stayed in `AppState.response_channels`
  until the next session-register-time cleanup pass. Long-running servers
  with repeated timeouts grew the map indefinitely. `send_to_plugin` now
  calls `cleanup_expired` opportunistically at the start of every dispatch.
- **Plugin registration spin loop**: when the studiolink server was offline,
  the plugin's initial `while not registered do … task.wait(5) end` loop ran
  forever. Bounded to 50 attempts (~4 minutes), then exits with a warn so
  the plugin doesn't hang Studio.
- **Screenshot 20 MB cap → 50 MB**: 5K (5120×2880) PNGs can hit 12+ MB raw,
  base64 pushes them past the old cap. Realistic captures from external
  displays were being rejected.

### Verified false-positive (no change)
- Server-context tool blocking (B1) — `executeServerTool` is already wrapped
  in `task.spawn` at line 294 of Main.server.luau. Audit agent missed it.
- `Plugin.Unloading` missing in Server context (B3) — Server context
  unregisters on `start_stop_play(stop)` via `task.defer`, which is the
  reliable path; the Edit-context Unloading hook covers Studio quits.

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
