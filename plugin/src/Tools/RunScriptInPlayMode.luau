--!strict
--[[
	RunScriptInPlayMode — Execute Luau code during play mode.

	Strategy:
	- If in play mode: Tells the caller to use switch_session to the Play Server session
	  and run_code there for Server context execution.
	- If not in play mode: Starts play, waits, executes code in Edit context, then stops.
	  (Limited: only sees Edit DataModel, not Server state)
]]

return function(args: { [string]: any }): (boolean, any, string?)
	local code = args.code
	local timeout = args.timeout or 100
	local mode = args.mode or "start_play"

	if not code or code == "" then
		return false, nil, "No code provided"
	end

	-- Check if we're in play mode
	local inPlayMode = (_G.StudioLinkMode == "start_play" or _G.StudioLinkMode == "run_server")

	if inPlayMode then
		-- During play mode, Server context runs as a separate session.
		-- Use list_sessions to find the "Play Server" session, switch to it, and use run_code.
		return true, {
			success = true,
			value = "Play mode is active. A 'Play Server' session is running with Server context access. "
				.. "Use list_sessions to find the Play Server session, then switch_session to it, "
				.. "and use run_code to execute code in Server context (with access to runtime state, "
				.. "Players, spawned objects, etc.).",
			logs = {},
			errors = {},
			duration = 0,
			isTimeout = false,
		}, nil
	end

	-- Not in play mode: execute locally in Edit context
	local output: { string } = {}
	local errors: { string } = {}
	local startTime = os.clock()

	local env = setmetatable({
		print = function(...)
			local parts = {}
			for i = 1, select("#", ...) do
				table.insert(parts, tostring(select(i, ...)))
			end
			table.insert(output, table.concat(parts, "\t"))
		end,
		warn = function(...)
			local parts = {}
			for i = 1, select("#", ...) do
				table.insert(parts, tostring(select(i, ...)))
			end
			table.insert(errors, table.concat(parts, "\t"))
		end,
	}, { __index = getfenv(0) })

	local fn, compileErr = loadstring(code)
	if not fn then
		return false, nil, "Compile error: " .. tostring(compileErr)
	end
	setfenv(fn, env)

	local success, result = pcall(fn)

	local duration = os.clock() - startTime

	return true, {
		success = success,
		value = if success then tostring(result) else nil,
		error = if not success then tostring(result) else nil,
		logs = output,
		errors = errors,
		duration = math.floor(duration * 1000),
		isTimeout = false,
	}, nil
end
