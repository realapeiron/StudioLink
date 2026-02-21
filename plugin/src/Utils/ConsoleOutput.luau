--!strict
-- ConsoleOutput: Capture and manage console output

local ConsoleOutput = {}

local outputBuffer: { string } = {}
local MAX_BUFFER = 500

function ConsoleOutput.capture(message: string, level: string?)
	local prefix = if level then "[" .. level .. "] " else "[OUTPUT] "
	table.insert(outputBuffer, prefix .. message)
	if #outputBuffer > MAX_BUFFER then
		table.remove(outputBuffer, 1)
	end
end

function ConsoleOutput.getAll(): string
	return table.concat(outputBuffer, "\n")
end

function ConsoleOutput.clear()
	table.clear(outputBuffer)
end

function ConsoleOutput.getAndClear(): string
	local output = ConsoleOutput.getAll()
	ConsoleOutput.clear()
	return output
end

return ConsoleOutput
