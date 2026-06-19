--!strict
-- Profiler: ScriptProfiler integration for performance analysis

local Profiler = {}
-- Keyed by __caller_id so two chats sharing one Studio session don't clobber
-- each other's in-flight profile.
local lastProfileData: { [string]: any } = {}

function Profiler.start(args: { [string]: any }): (boolean, any, string?)
	-- This is not a per-script CPU profiler (that needs ScriptProfiler /
	-- MicroProfiler — see microprofiler_capture). It snapshots engine Stats at
	-- start so profile_stop can report what changed over the interval.
	local callerId = args.__caller_id or "default"
	local ok, err = pcall(function()
		local Stats = game:GetService("Stats")
		lastProfileData[callerId] = {
			startTime = os.clock(),
			startInstanceCount = Stats.InstanceCount,
			running = true,
		}
	end)

	if ok then
		return true, {
			message = "Stats capture started",
			note = "Engine Stats over the interval — call profile_stop to read the delta. "
				.. "For a per-script CPU profile use microprofiler_capture.",
		}, nil
	else
		return false, nil, "Failed to start profiler: " .. tostring(err)
	end
end

function Profiler.stop(args: { [string]: any }): (boolean, any, string?)
	local callerId = args.__caller_id or "default"
	local data = lastProfileData[callerId]
	if not data or not data.running then
		return false, nil, "Profiler is not running"
	end

	local duration = os.clock() - data.startTime
	data.running = false
	data.duration = duration

	-- Report the interval: InstanceCount as a start→end delta, the rest as
	-- instantaneous end-of-interval readings (Stats exposes no history).
	local Stats = game:GetService("Stats")
	local endInstanceCount = Stats.InstanceCount
	local statsData = {
		duration = math.floor(duration * 1000) / 1000,
		instanceCountStart = data.startInstanceCount,
		instanceCountEnd = endInstanceCount,
		instanceCountDelta = endInstanceCount - (data.startInstanceCount or endInstanceCount),
		heartbeatTimeMs = Stats.HeartbeatTimeMs,
		physicsStepTimeMs = Stats.PhysicsStepTimeMs,
		movingPrimitivesCount = Stats.MovingPrimitivesCount,
		primitivesCount = Stats.PrimitivesCount,
		note = "instanceCountDelta spans the interval; heartbeat/physics are end-of-interval snapshots.",
	}

	return true, statsData, nil
end

function Profiler.analyze(args: { [string]: any }): (boolean, any, string?)
	-- Analyze current performance state
	local Stats = game:GetService("Stats")
	local analysis: { any } = {}
	local suggestions: { string } = {}

	-- Instance count analysis
	local instanceCount = Stats.InstanceCount
	if instanceCount > 100000 then
		table.insert(suggestions, "CRITICAL: " .. instanceCount .. " instances — consider streaming or reducing scene complexity")
	elseif instanceCount > 50000 then
		table.insert(suggestions, "WARNING: " .. instanceCount .. " instances — approaching performance limits")
	end

	-- Heartbeat analysis
	local heartbeat = Stats.HeartbeatTimeMs
	if heartbeat > 16.67 then
		table.insert(suggestions, "Heartbeat time " .. string.format("%.1f", heartbeat) .. "ms — below 60 FPS target (16.67ms)")
	end

	-- Physics analysis
	local physicsTime = Stats.PhysicsStepTimeMs
	if physicsTime > 8 then
		table.insert(suggestions, "Physics step time " .. string.format("%.1f", physicsTime) .. "ms — consider reducing physics objects or using constraints")
	end

	-- Moving primitives analysis
	local movingPrims = Stats.MovingPrimitivesCount
	if movingPrims > 500 then
		table.insert(suggestions, movingPrims .. " moving primitives — consider anchoring static parts")
	end

	-- Script analysis through code patterns
	local TreeWalker = require(script.Parent.Parent.Utils.TreeWalker)
	local scripts = TreeWalker.collectScripts()
	local scriptIssues: { any } = {}

	for _, scriptInstance in ipairs(scripts) do
		local ok, source = pcall(function() return (scriptInstance :: any).Source end)
		if not ok or not source then continue end

		-- Check for performance anti-patterns
		if source:find("FindFirstChild") then
			local count = 0
			for _ in source:gmatch("FindFirstChild") do count += 1 end
			if count > 10 then
				table.insert(scriptIssues, {
					script = scriptInstance:GetFullName(),
					issue = count .. " FindFirstChild calls — cache results in variables",
				})
			end
		end

		if source:find("GetChildren%(%)") or source:find("GetDescendants%(%)") then
			if source:find("Heartbeat") or source:find("RenderStepped") then
				table.insert(scriptIssues, {
					script = scriptInstance:GetFullName(),
					issue = "GetChildren/GetDescendants in render loop — extremely expensive, cache results",
				})
			end
		end

		if source:find("CFrame") then
			local count = 0
			for _ in source:gmatch("CFrame%.new") do count += 1 end
			if count > 20 then
				table.insert(scriptIssues, {
					script = scriptInstance:GetFullName(),
					issue = count .. " CFrame.new allocations — consider reusing CFrame values",
				})
			end
		end
	end

	return true, {
		stats = {
			instanceCount = instanceCount,
			heartbeatTimeMs = heartbeat,
			physicsStepTimeMs = physicsTime,
			movingPrimitives = movingPrims,
			totalPrimitives = Stats.PrimitivesCount,
		},
		suggestions = suggestions,
		scriptIssues = scriptIssues,
		overallHealth = if #suggestions == 0 then "Good" elseif #suggestions <= 2 then "Fair" else "Needs Optimization",
	}, nil
end

return Profiler
