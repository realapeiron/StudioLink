--!strict
-- RunCode: Execute Luau code in Studio context

local ConsoleOutput = require(script.Parent.Parent.Utils.ConsoleOutput)

return function(args: { [string]: any }): (boolean, any, string?)
	local command = args.command
	if not command or command == "" then
		return false, nil, "No command provided"
	end

	ConsoleOutput.clear()

	-- Override print/warn/error to capture output
	local output: { string } = {}
	local env = setmetatable({
		print = function(...)
			local parts = {}
			for i = 1, select("#", ...) do
				table.insert(parts, tostring(select(i, ...)))
			end
			local line = table.concat(parts, "\t")
			table.insert(output, "[OUTPUT] " .. line)
		end,
		warn = function(...)
			local parts = {}
			for i = 1, select("#", ...) do
				table.insert(parts, tostring(select(i, ...)))
			end
			local line = table.concat(parts, "\t")
			table.insert(output, "[WARNING] " .. line)
		end,
		error = function(msg, level)
			table.insert(output, "[ERROR] " .. tostring(msg))
		end,
	}, { __index = getfenv(0) })

	local fn, compileError = loadstring(command)
	if not fn then
		return false, nil, "Compile error: " .. tostring(compileError)
	end

	setfenv(fn, env)
	local success, result = pcall(fn)

	local outputStr = table.concat(output, "\n")

	if success then
		if result ~= nil then
			outputStr = outputStr .. "\n[RETURNED RESULTS] " .. tostring(result)
		end
		return true, outputStr, nil
	else
		return false, outputStr, tostring(result)
	end
end
