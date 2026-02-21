--!strict
--[[
	StudioLink — Advanced Roblox Studio MCP Server Plugin
	36 tools for professional game development with AI assistance

	This plugin communicates with the StudioLink Rust server via HTTP long polling.
	It receives tool commands, executes them, and returns results.

	Multi-place support: Each Studio instance registers as a unique session.
	The Rust server routes tool calls to the active session.
]]

local RunService = game:GetService("RunService")

-- During play test, the plugin loads in 3 contexts: Edit, Server, Client
-- Client: Can't use HttpService → skip entirely
-- Server: Runs its own HTTP polling session (registers as "Play Server" session)
--         This allows MCP to run_code directly in Server context via switch_session
-- Edit: Run full plugin (HTTP polling, tool execution, etc.)
if RunService:IsRunning() then
	if RunService:IsClient() then
		return -- Client can't use HttpService, skip
	end

	-- ═══════════════════════════════════════════
	-- SERVER CONTEXT: Full HTTP polling with own session
	-- ═══════════════════════════════════════════
	local HttpService = game:GetService("HttpService")
	local SERVER_URL = "http://127.0.0.1:34872"
	local SERVER_SESSION_ID = HttpService:GenerateGUID(false)
	local Serializer = require(script.Parent.Utils.Serializer)

	-- Store plugin ref for stop
	local pluginRef = _G.StudioLinkPlugin or plugin
	_G.StudioLinkPlugin = pluginRef

	-- HTTP helper (same as Edit context)
	local function httpRequest(method: string, endpoint: string, body: any?): (boolean, any)
		local url = SERVER_URL .. endpoint
		local requestData: any = {
			Url = url,
			Method = method,
			Headers = { ["Content-Type"] = "application/json" },
		}
		if body then
			requestData.Body = HttpService:JSONEncode(body)
		end
		local ok, response = pcall(function()
			return HttpService:RequestAsync(requestData)
		end)
		if not ok then return false, response end
		if response.StatusCode == 200 then
			local decodeOk, decoded = pcall(function()
				return HttpService:JSONDecode(response.Body)
			end)
			return decodeOk and true or true, decodeOk and decoded or response.Body
		elseif response.StatusCode == 204 then
			return false, "no_content"
		else
			return false, "HTTP " .. response.StatusCode
		end
	end

	-- Register Server tools (subset that makes sense in Server context)
	local ServerTools: { [string]: any } = {}

	-- Server run_code: uses dynamic Script injection (loadstring not available in Server context)
	ServerTools["run_code"] = function(args: { [string]: any })
		local code = args.command or args.code or ""
		if code == "" then
			return false, nil, "No code provided"
		end

		-- Clean up any previous runner
		local RS = game:GetService("ReplicatedStorage")
		local oldResult = RS:FindFirstChild("__StudioLinkResult")
		if oldResult then oldResult:Destroy() end
		local SSS = game:GetService("ServerScriptService")
		local oldRunner = SSS:FindFirstChild("__StudioLinkRunner")
		if oldRunner then oldRunner:Destroy() end

		-- Create a Script that wraps user code with output capture
		-- The Script stores its result in a StringValue in ReplicatedStorage
		local wrappedCode = [[
local __output = {}
local __origPrint = print
local __origWarn = warn
print = function(...)
	local parts = {}
	for i = 1, select("#", ...) do
		table.insert(parts, tostring(select(i, ...)))
	end
	table.insert(__output, table.concat(parts, "\t"))
end
warn = function(...)
	local parts = {}
	for i = 1, select("#", ...) do
		table.insert(parts, tostring(select(i, ...)))
	end
	table.insert(__output, table.concat(parts, "\t"))
end

local __ok, __err = pcall(function()
]] .. code .. [[

end)

print = __origPrint
warn = __origWarn

local __resultStr = ""
if __ok then
	__resultStr = "[OUTPUT] " .. table.concat(__output, "\n[OUTPUT] ")
else
	__resultStr = "[ERROR] " .. tostring(__err) .. "\n[OUTPUT] " .. table.concat(__output, "\n[OUTPUT] ")
end
if #__output == 0 and __ok then
	__resultStr = "[OUTPUT] (no output)"
elseif #__output == 0 and not __ok then
	__resultStr = "[ERROR] " .. tostring(__err)
end

local marker = Instance.new("StringValue")
marker.Name = "__StudioLinkResult"
marker.Value = __resultStr
marker.Parent = game:GetService("ReplicatedStorage")
]]

		local runnerScript = Instance.new("Script")
		runnerScript.Name = "__StudioLinkRunner"
		;(runnerScript :: any).Source = wrappedCode
		runnerScript.Parent = SSS

		-- Wait for result (Script auto-runs when parented)
		local startTime = os.clock()
		local timeoutSec = 15
		while os.clock() - startTime < timeoutSec do
			local marker = RS:FindFirstChild("__StudioLinkResult")
			if marker and marker:IsA("StringValue") and marker.Value ~= "" then
				local resultStr = marker.Value
				marker:Destroy()
				runnerScript:Destroy()
				return true, resultStr, nil
			end
			task.wait(0.2)
		end

		-- Timeout cleanup
		local marker2 = RS:FindFirstChild("__StudioLinkResult")
		if marker2 then marker2:Destroy() end
		runnerScript:Destroy()
		return false, nil, "Script execution timed out (15s)"
	end

	ServerTools["get_console_output"] = require(script.Parent.Tools.GetConsoleOutput)
	ServerTools["get_studio_mode"] = function()
		return true, "start_play (server context)", nil
	end

	-- Stop tool for Server context (can call EndTest directly)
	ServerTools["start_stop_play"] = function(args: { [string]: any })
		local mode = args.mode or "stop"
		if mode == "stop" then
			local sts = game:FindFirstChildOfClass("StudioTestService") :: any
			if sts then
				pcall(function() sts:EndTest({}) end)
				return true, "Stopped from Server context", nil
			end
			return false, nil, "StudioTestService not found"
		end
		return false, nil, "Only 'stop' is supported from Server context"
	end

	-- Execute tool in Server context
	local function executeServerTool(request: any): any
		local toolName = request.tool
		local args = request.args or {}
		local id = request.id
		local handler = ServerTools[toolName]
		if not handler then
			return {
				id = id, success = false, result = "null",
				error = "Tool '" .. tostring(toolName) .. "' not available in Server context. Available: run_code, get_console_output, get_studio_mode, start_stop_play",
			}
		end
		local ok, success, result, err = pcall(handler, args)
		if not ok then
			return {
				id = id, success = false, result = "null",
				error = "Server tool error: " .. tostring(success),
			}
		end
		local serializedResult = Serializer.serialize(result)
		if serializedResult == nil then serializedResult = "null" end
		return {
			id = id, success = success, result = serializedResult, error = err,
		}
	end

	-- Get place info for Server session registration
	local placeId = game.PlaceId or 0
	local placeName = "Play Server"
	pcall(function()
		local info = game:GetService("MarketplaceService"):GetProductInfo(placeId)
		if info and info.Name then
			placeName = info.Name .. " (Play Server)"
		end
	end)

	-- Register Server session
	print("[StudioLink-Server] Registering play server session: " .. SERVER_SESSION_ID)
	local registered = false
	for _ = 1, 10 do
		local regOk, regResult = httpRequest("POST", "/register", {
			session_id = SERVER_SESSION_ID,
			place_id = placeId,
			place_name = placeName,
			game_id = game.GameId or 0,
		})
		if regOk and type(regResult) == "table" and regResult.status == "registered" then
			registered = true
			print("[StudioLink-Server] Play server session registered!")
			break
		end
		task.wait(1)
	end

	if not registered then
		warn("[StudioLink-Server] Failed to register, stopping")
		return
	end

	-- Also keep the stop listener (via plugin settings from Edit context)
	task.spawn(function()
		while RunService:IsRunning() do
			local ok, stopRequested = pcall(function()
				return pluginRef:GetSetting("StudioLink_StopRequested")
			end)
			if ok and stopRequested == true then
				pcall(function() pluginRef:SetSetting("StudioLink_StopRequested", false) end)
				local sts = game:FindFirstChildOfClass("StudioTestService") :: any
				if sts then
					pcall(function() sts:EndTest({}) end)
				end
				break
			end
			task.wait(0.3)
		end
	end)

	-- Server context polling loop
	while RunService:IsRunning() do
		local pollOk, request = httpRequest("GET", "/request?session_id=" .. SERVER_SESSION_ID)
		if pollOk and request and type(request) == "table" and request.id then
			print("[StudioLink-Server] Executing: " .. tostring(request.tool))
			local response = executeServerTool(request)
			httpRequest("POST", "/response", response)
		else
			task.wait(0.5)
		end
	end

	-- Unregister on play end
	pcall(function()
		httpRequest("POST", "/unregister", { session_id = SERVER_SESSION_ID })
		print("[StudioLink-Server] Session unregistered")
	end)
	return
end

local HttpService = game:GetService("HttpService")

-- Configuration
local SERVER_PORT = 34872
local SERVER_URL = "http://127.0.0.1:" .. SERVER_PORT
local POLL_INTERVAL = 0.5 -- seconds between poll retries on error
local REGISTER_RETRY_INTERVAL = 3 -- seconds between registration retries
local VERSION = "0.1.0"

-- Generate a unique session ID for this Studio instance
local SESSION_ID = HttpService:GenerateGUID(false)

-- Store plugin reference globally for tools that need it
_G.StudioLinkPlugin = plugin

-- Tool registry
local Tools: { [string]: any } = {}

-- Core tools (Faz 1)
Tools["run_code"] = require(script.Parent.Tools.RunCode)
Tools["insert_model"] = require(script.Parent.Tools.InsertModel)
Tools["get_console_output"] = require(script.Parent.Tools.GetConsoleOutput)
Tools["start_stop_play"] = require(script.Parent.Tools.StartStopPlay)
Tools["run_script_in_play_mode"] = require(script.Parent.Tools.RunScriptInPlayMode)
Tools["get_studio_mode"] = require(script.Parent.Tools.GetStudioMode)

-- DataStore tools (Faz 2)
local DataStoreDebug = require(script.Parent.Tools.DataStoreDebug)
Tools["datastore_list"] = function(args) return DataStoreDebug.list(args) end
Tools["datastore_get"] = function(args) return DataStoreDebug.get(args) end
Tools["datastore_set"] = function(args) return DataStoreDebug.set(args) end
Tools["datastore_delete"] = function(args) return DataStoreDebug.delete(args) end
Tools["datastore_scan"] = function(args) return DataStoreDebug.scan(args) end

-- Profiler tools (Faz 2)
local Profiler = require(script.Parent.Tools.Profiler)
Tools["profile_start"] = function(args) return Profiler.start(args) end
Tools["profile_stop"] = function(args) return Profiler.stop(args) end
Tools["profile_analyze"] = function(args) return Profiler.analyze(args) end

-- Place diff tools (Faz 3)
local PlaceDiff = require(script.Parent.Tools.PlaceDiff)
Tools["snapshot_take"] = function(args) return PlaceDiff.take(args) end
Tools["snapshot_compare"] = function(args) return PlaceDiff.compare(args) end
Tools["snapshot_list"] = function(args) return PlaceDiff.list(args) end

-- Test tools (Faz 3)
local TestRunner = require(script.Parent.Tools.TestRunner)
Tools["test_run"] = function(args) return TestRunner.run(args) end
Tools["test_create"] = function(args) return TestRunner.create(args) end
Tools["test_report"] = function(args) return TestRunner.report(args) end

-- Security tools (Faz 4)
local SecurityAudit = require(script.Parent.Tools.SecurityAudit)
Tools["security_scan"] = function() return SecurityAudit.scan() end
Tools["security_report"] = function() return SecurityAudit.report() end

-- Analysis tools (Faz 4)
Tools["dependency_map"] = require(script.Parent.Tools.DependencyMap)
Tools["memory_scan"] = require(script.Parent.Tools.MemoryLeakScan)
Tools["lint_scripts"] = require(script.Parent.Tools.CodeAnalyzer)

-- Animation tools (Faz 5)
local AnimationInspect = require(script.Parent.Tools.AnimationInspect)
Tools["animation_list"] = function(args) return AnimationInspect.list(args) end
Tools["animation_inspect"] = function(args) return AnimationInspect.inspect(args) end
Tools["animation_conflicts"] = function(args) return AnimationInspect.conflicts(args) end

-- Network tools (Faz 5)
local NetworkMonitor = require(script.Parent.Tools.NetworkMonitor)
Tools["network_monitor_start"] = function(args) return NetworkMonitor.start(args) end
Tools["network_monitor_stop"] = function(args) return NetworkMonitor.stop(args) end

-- UI tools (Faz 5)
local UIInspector = require(script.Parent.Tools.UIInspector)
Tools["ui_tree"] = function(args) return UIInspector.tree(args) end
Tools["ui_analyze"] = function(args) return UIInspector.analyze(args) end

-- Documentation tools (Faz 5)
Tools["docs_generate"] = require(script.Parent.Tools.AutoDocs)

-- Serializer for responses
local Serializer = require(script.Parent.Utils.Serializer)

-- Count tools
local toolCount = 0
for _ in pairs(Tools) do toolCount += 1 end

print("[StudioLink] v" .. VERSION .. " — Loading " .. tostring(toolCount) .. " tools (Session: " .. SESSION_ID .. ")")

-- HTTP request helper
local function httpRequest(method: string, endpoint: string, body: any?): (boolean, any)
	local url = SERVER_URL .. endpoint
	local requestData: any = {
		Url = url,
		Method = method,
		Headers = {
			["Content-Type"] = "application/json",
		},
	}

	if body then
		requestData.Body = HttpService:JSONEncode(body)
	end

	local ok, response = pcall(function()
		return HttpService:RequestAsync(requestData)
	end)

	if not ok then
		return false, response
	end

	if response.StatusCode == 200 then
		local decodeOk, decoded = pcall(function()
			return HttpService:JSONDecode(response.Body)
		end)
		if decodeOk then
			return true, decoded
		else
			return true, response.Body
		end
	elseif response.StatusCode == 204 then
		-- No content (long poll timeout, normal)
		return false, "no_content"
	else
		return false, "HTTP " .. response.StatusCode
	end
end

-- ═══════════════════════════════════════════
-- SESSION REGISTRATION
-- ═══════════════════════════════════════════

-- Gather place information for registration
local function getPlaceInfo(): { [string]: any }
	local placeId = game.PlaceId or 0
	local placeName = "Unknown Place"

	-- Try to get the place name from MarketplaceService
	local ok, name = pcall(function()
		local MarketplaceService = game:GetService("MarketplaceService")
		local info = MarketplaceService:GetProductInfo(placeId)
		return info and info.Name or "Unknown Place"
	end)
	if ok and name then
		placeName = name
	end

	return {
		session_id = SESSION_ID,
		place_id = placeId,
		place_name = placeName,
		game_id = game.GameId or 0,
	}
end

-- Register this session with the StudioLink server
local function registerSession(): boolean
	local placeInfo = getPlaceInfo()
	local regOk, result = httpRequest("POST", "/register", placeInfo)

	if regOk and type(result) == "table" and result.status == "registered" then
		print("[StudioLink] Session registered successfully: " .. SESSION_ID)
		print("[StudioLink] Place: " .. placeInfo.place_name .. " (ID: " .. tostring(placeInfo.place_id) .. ")")
		return true
	else
		warn("[StudioLink] Failed to register session: " .. tostring(result))
		return false
	end
end

-- Unregister this session from the server
local function unregisterSession()
	local unregOk, _ = httpRequest("POST", "/unregister", {
		session_id = SESSION_ID,
	})
	if unregOk then
		print("[StudioLink] Session unregistered: " .. SESSION_ID)
	else
		warn("[StudioLink] Failed to unregister session (server may be offline)")
	end
end

-- Handle plugin unload — clean up session
plugin.Unloading:Connect(function()
	print("[StudioLink] Plugin unloading, cleaning up...")
	unregisterSession()
end)

-- ═══════════════════════════════════════════
-- TOOL EXECUTION
-- ═══════════════════════════════════════════

-- Execute a tool and return the response
local function executeTool(request: any): any
	local toolName = request.tool
	local args = request.args or {}
	local id = request.id

	local handler = Tools[toolName]
	if not handler then
		return {
			id = id,
			success = false,
			result = "null",
			error = "Unknown tool: " .. tostring(toolName),
		}
	end

	local ok, success, result, err
	if type(handler) == "function" then
		ok, success, result, err = pcall(handler, args)
	else
		ok = false
		err = "Tool handler is not a function"
	end

	if not ok then
		return {
			id = id,
			success = false,
			result = "null",
			error = "Tool execution error: " .. tostring(success), -- pcall error is in 'success'
		}
	end

	-- Serialize the result (ensure never nil — nil causes JSON field to be omitted, breaking Rust deserialization)
	local serializedResult = Serializer.serialize(result)
	if serializedResult == nil then
		serializedResult = "null"
	end

	return {
		id = id,
		success = success,
		result = serializedResult,
		error = err,
	}
end

-- ═══════════════════════════════════════════
-- MAIN LOOP: Register + Poll
-- ═══════════════════════════════════════════

print("[StudioLink] Starting session-aware polling on " .. SERVER_URL)

-- Step 1: Register with retries until server is available
local registered = false
while not registered do
	registered = registerSession()
	if not registered then
		print("[StudioLink] Server not available, retrying in " .. tostring(REGISTER_RETRY_INTERVAL) .. "s...")
		task.wait(REGISTER_RETRY_INTERVAL)
	end
end

-- Step 2: Main polling loop with session_id
while true do
	-- Poll for next request, scoped to this session
	local pollOk, request = httpRequest("GET", "/request?session_id=" .. SESSION_ID)

	if pollOk and request and type(request) == "table" and request.id then
		-- Got a request, execute the tool
		print("[StudioLink] Executing tool: " .. tostring(request.tool))

		local response = executeTool(request)

		-- Send response back
		local sendOk, sendResult = httpRequest("POST", "/response", response)
		if not sendOk then
			warn("[StudioLink] Failed to send response: " .. tostring(sendResult))
		end
	else
		-- No request or error, wait briefly before next poll
		task.wait(POLL_INTERVAL)
	end
end
