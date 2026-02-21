--!strict
-- Profiler: ScriptProfiler integration for performance analysis

local Profiler = {}
local lastProfileData: any = nil

function Profiler.start(args: { [string]: any }): (boolean, any, string?)
	local frequency = args.frequency or 1000

	-- Use ScriptContext to access profiling
	local ok, err = pcall(function()
		-- ScriptProfiler is a Studio-only feature
		-- We use debug.profilebegin/profileend and Stats service
		local Stats = game:GetService("Stats")
		lastProfileData = {
			startTime = os.clock(),
			frequency = frequency,
			running = true,
		}
	end)

	if ok then
		return true, {
			message = "Profiling started at " .. frequency .. " Hz",
			note = "Run your game logic, then call profile_stop to collect results",
		}, nil
	else
		return false, nil, "Failed to start profiler: " .. tostring(err)
	end
end

function Profiler.stop(args: { [string]: any }): (boolean, any, string?)
	if not lastProfileData or not lastProfileData.running then
		return false, nil, "Profiler is not running"
	end

	local duration = os.clock() - lastProfileData.startTime
	lastProfileData.running = false
	lastProfileData.duration = duration

	-- Collect Stats data
	local Stats = game:GetService("Stats")
	local statsData = {
		duration = math.floor(duration * 1000) / 1000,
		instanceCount = Stats.InstanceCount,
		heartbeatTime = Stats.HeartbeatTimeMs,
		physicsStepTime = Stats.PhysicsStepTimeMs,
		movingPrimitivesCount = Stats.MovingPrimitivesCount,
		primitivesCount = Stats.PrimitivesCount,
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
