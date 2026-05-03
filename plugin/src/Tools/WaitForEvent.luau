--!strict
local PlayHelpers = require(script.Parent.Parent.Utils.PlayHelpers)
local PathResolver = require(script.Parent.Parent.Utils.PathResolver)

return function(args: { [string]: any }): (boolean, any, string?)
	local ctxOk, ctxErr = PlayHelpers.requireContext("play")
	if not ctxOk then
		return false, nil, ctxErr
	end

	local path = args.instance_path
	local eventName = args.event_name
	if type(path) ~= "string" or type(eventName) ~= "string" then
		return false, nil, "instance_path and event_name are required strings"
	end

	local timeoutSecs: number = if type(args.timeout_secs) == "number" then args.timeout_secs else 30
	if timeoutSecs > 110 then
		timeoutSecs = 110
	end
	local captureArgs: boolean = if args.capture_args == nil then true else args.capture_args

	local instance = PathResolver.resolve(path)
	if not instance then
		return false, nil, "instance not found: " .. path
	end

	local event = (instance :: any)[eventName]
	if event == nil then
		return false, nil, "no event named '" .. eventName .. "' on " .. path
	end

	local fired = false
	local capturedArgs: { string } = {}
	local startTime = os.clock()

	local conn: RBXScriptConnection? = nil
	local connectOk, connectErr = pcall(function()
		conn = (event :: any):Connect(function(...)
			fired = true
			if captureArgs then
				local n = select("#", ...)
				for i = 1, n do
					capturedArgs[i] = tostring(select(i, ...))
				end
			end
		end)
	end)

	if not connectOk then
		return false, nil, "could not connect to '" .. eventName .. "': " .. tostring(connectErr)
	end

	local deadline = os.clock() + timeoutSecs
	while not fired and os.clock() < deadline do
		task.wait(0.05)
	end

	if conn then
		conn:Disconnect()
	end

	return true,
		{
			fired = fired,
			elapsed_ms = math.floor((os.clock() - startTime) * 1000),
			timeout = not fired,
			captured_args = if fired and captureArgs then capturedArgs else nil,
		},
		nil
end
