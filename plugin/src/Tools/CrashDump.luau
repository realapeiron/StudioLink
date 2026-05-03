--!strict
-- crash_dump — collect recent log activity within a time window and isolate
-- the error subset. Studio process crashes are NOT capturable from a plugin
-- (no access to .dmp files) — this tool reports *logical* crashes (script
-- errors, recent log noise) only.

local function typeName(mt: any): string
	return tostring(mt):gsub("Enum%.MessageType%.Message", "")
end

return function(args: { [string]: any }): (boolean, any, string?)
	local LogService = game:GetService("LogService")
	local history = LogService:GetLogHistory()

	local windowSecs: number = if type(args.window_secs) == "number" then args.window_secs else 30
	if windowSecs <= 0 then
		return false, nil, "window_secs must be > 0"
	end

	local now = os.time()
	local cutoff = now - windowSecs

	local recent: { any } = {}
	local errors: { any } = {}
	local stackTraces: { string } = {}

	for _, entry in ipairs(history) do
		if entry.timestamp >= cutoff then
			local item = {
				message = entry.message,
				type = typeName(entry.messageType),
				timestamp = entry.timestamp,
			}
			table.insert(recent, item)
			if entry.messageType == Enum.MessageType.MessageError then
				table.insert(errors, item)
				-- Stack frame heuristics: "Stack Begin" Roblox marker or
				-- "<file>:<line>" lines.
				local msg: string = entry.message
				if msg:find("Stack Begin") or msg:find("\n%s+%S+:%d+") then
					table.insert(stackTraces, msg)
				end
			end
		end
	end

	return true,
		{
			window_secs = windowSecs,
			now_unix = now,
			recent_count = #recent,
			recent_messages = recent,
			errors = errors,
			error_count = #errors,
			stack_traces = stackTraces,
			note = "Studio process crashes (.dmp) are not accessible from plugin context. This tool covers logical errors only.",
		},
		nil
end
