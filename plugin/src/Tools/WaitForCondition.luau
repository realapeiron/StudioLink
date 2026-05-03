--!strict
local PlayHelpers = require(script.Parent.Parent.Utils.PlayHelpers)
local PathResolver = require(script.Parent.Parent.Utils.PathResolver)

local function compare(value: any, op: string, target: any): boolean
	if op == "==" then
		return value == target
	elseif op == "!=" then
		return value ~= target
	elseif type(value) ~= "number" or type(target) ~= "number" then
		return false
	elseif op == ">" then
		return value > target
	elseif op == ">=" then
		return value >= target
	elseif op == "<" then
		return value < target
	elseif op == "<=" then
		return value <= target
	end
	return false
end

return function(args: { [string]: any }): (boolean, any, string?)
	local ctxOk, ctxErr = PlayHelpers.requireContext("play")
	if not ctxOk then
		return false, nil, ctxErr
	end

	local path = args.instance_path
	local prop = args.property
	if type(path) ~= "string" or type(prop) ~= "string" then
		return false, nil, "instance_path and property are required strings"
	end

	local op: string = args.operator or "=="
	local valid = { ["=="] = true, ["!="] = true, [">"] = true, [">="] = true, ["<"] = true, ["<="] = true }
	if not valid[op] then
		return false, nil, "operator must be one of: ==, !=, >, >=, <, <="
	end

	local target = args.target
	local pollMs: number = if type(args.poll_interval_ms) == "number" then args.poll_interval_ms else 100
	local timeoutSecs: number = if type(args.timeout_secs) == "number" then args.timeout_secs else 30
	if timeoutSecs > 110 then
		timeoutSecs = 110 -- below EXTENDED_TIMEOUT (120s) on the server side
	end

	local instance = PathResolver.resolve(path)
	if not instance then
		return false, nil, "instance not found: " .. path
	end

	local startTime = os.clock()
	local deadline = startTime + timeoutSecs
	local lastValue: any = nil
	local lastError: string? = nil

	while os.clock() < deadline do
		local readOk, val = pcall(function()
			return (instance :: any)[prop]
		end)
		if not readOk then
			lastError = "could not read property '" .. prop .. "': " .. tostring(val)
			break
		end
		lastValue = val
		if compare(val, op, target) then
			return true,
				{
					satisfied = true,
					last_value = val,
					elapsed_ms = math.floor((os.clock() - startTime) * 1000),
				},
				nil
		end
		task.wait(pollMs / 1000)
	end

	return true,
		{
			satisfied = false,
			last_value = lastValue,
			elapsed_ms = math.floor((os.clock() - startTime) * 1000),
			timeout = lastError == nil,
			error = lastError,
		},
		nil
end
