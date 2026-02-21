--!strict
-- Formatter: Output formatting utilities

local Formatter = {}

function Formatter.formatTable(tbl: { [string]: any }, indent: string?): string
	indent = indent or ""
	local lines = {}
	for key, value in pairs(tbl) do
		if type(value) == "table" then
			table.insert(lines, indent .. tostring(key) .. ":")
			table.insert(lines, Formatter.formatTable(value, indent .. "  "))
		else
			table.insert(lines, indent .. tostring(key) .. " = " .. tostring(value))
		end
	end
	return table.concat(lines, "\n")
end

function Formatter.formatList(items: { string }, header: string?): string
	local lines = {}
	if header then
		table.insert(lines, "=== " .. header .. " ===")
	end
	for i, item in ipairs(items) do
		table.insert(lines, i .. ". " .. item)
	end
	return table.concat(lines, "\n")
end

function Formatter.formatIssue(severity: string, location: string, message: string): string
	return string.format("[%s] %s: %s", severity, location, message)
end

return Formatter
