--!strict
-- microprofiler_capture — wrap a Luau code block in debug.profilebegin/end
-- and measure wall time + Lua heap delta. Studio's MicroProfiler GUI export
-- is not exposed; this tool covers script-level metrics only.

return function(args: { [string]: any }): (boolean, any, string?)
	local code = args.code
	if type(code) ~= "string" or code == "" then
		return false, nil, "code is required (string)"
	end
	local label = if type(args.label) == "string" then args.label else "studiolink_capture"

	local fn, compileError = loadstring(code)
	if not fn then
		return false, nil, "compile error: " .. tostring(compileError)
	end

	debug.profilebegin(label)
	local startCpu = os.clock()
	local startMem = collectgarbage("count") -- KB
	local ok, result = pcall(fn)
	local endCpu = os.clock()
	local endMem = collectgarbage("count")
	debug.profileend()

	local elapsedMs = (endCpu - startCpu) * 1000
	local memDeltaKb = endMem - startMem

	return true,
		{
			label = label,
			elapsed_ms = elapsedMs,
			memory_delta_kb = memDeltaKb,
			success = ok,
			result = if ok then tostring(result) else nil,
			error = if not ok then tostring(result) else nil,
			note = "Studio's MicroProfiler GUI export is not exposed by Roblox APIs. "
				.. "Wall time uses os.clock(), heap delta uses collectgarbage('count'). "
				.. "Script-level only — no per-frame Render/Physics/Network breakdown.",
		},
		nil
end
