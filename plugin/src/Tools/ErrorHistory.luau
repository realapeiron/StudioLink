--!strict
-- error_history — read LogService:GetLogHistory() with optional filtering by
-- message type and substring pattern. Returns up to `limit` most-recent
-- matching entries (default 100).

local TYPE_MAP: { [string]: any } = {
	Output = Enum.MessageType.MessageOutput,
	Info = Enum.MessageType.MessageInfo,
	Warning = Enum.MessageType.MessageWarning,
	Error = Enum.MessageType.MessageError,
}

local function typeName(mt: any): string
	return tostring(mt):gsub("Enum%.MessageType%.Message", "")
end

return function(args: { [string]: any }): (boolean, any, string?)
	local LogService = game:GetService("LogService")
	local history = LogService:GetLogHistory()

	local messageType = args.message_type
	local filterType: any = nil
	if type(messageType) == "string" then
		filterType = TYPE_MAP[messageType]
		if not filterType then
			return false, nil, "message_type must be Output, Info, Warning, or Error"
		end
	end

	local pattern = if type(args.pattern) == "string" then args.pattern else nil
	local limit: number = if type(args.limit) == "number" then args.limit else 100

	local result: { { message: string, type: string, timestamp: number } } = {}
	-- Newest first.
	for i = #history, 1, -1 do
		local entry = history[i]
		if filterType and entry.messageType ~= filterType then
			continue
		end
		if pattern and not (entry.message :: string):find(pattern, 1, true) then
			continue
		end
		table.insert(result, {
			message = entry.message,
			type = typeName(entry.messageType),
			timestamp = entry.timestamp,
		})
		if #result >= limit then
			break
		end
	end

	return true,
		{
			entries = result,
			total_in_history = #history,
			filtered_count = #result,
		},
		nil
end
