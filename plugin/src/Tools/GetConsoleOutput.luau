--!strict
local ConsoleOutput = require(script.Parent.Parent.Utils.ConsoleOutput)

return function(args: { [string]: any }): (boolean, any, string?)
	local output = ConsoleOutput.getAndClear()
	if output == "" then
		return true, "No console output captured", nil
	end
	return true, output, nil
end
