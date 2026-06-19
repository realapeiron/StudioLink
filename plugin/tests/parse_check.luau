--!strict
-- Syntax (parse) check for every .luau file under plugin/src, runnable without
-- a Roblox runtime. Catches missing `end`, unbalanced parens, bad commas, etc.
-- It does NOT type-check or resolve Roblox globals. Run from the repo root:
--   lune run plugin/tests/parse_check.luau

local fs = require("@lune/fs")

local function walk(dir: string, acc: { string }): { string }
	for _, entry in ipairs(fs.readDir(dir)) do
		local path = dir .. "/" .. entry
		if fs.metadata(path).kind == "dir" then
			walk(path, acc)
		elseif entry:match("%.luau$") then
			table.insert(acc, path)
		end
	end
	return acc
end

local files = walk("plugin/src", {})
table.sort(files)

local errors = 0
for _, path in ipairs(files) do
	local src = fs.readFile(path)
	local fn, err = loadstring(src)
	if fn == nil then
		errors += 1
		print("  SYNTAX ERROR: " .. path)
		print("    " .. tostring(err))
	end
end

print(string.format("%d files parsed, %d syntax error(s)", #files, errors))
if errors > 0 then
	error(tostring(errors) .. " file(s) failed to parse")
end
print("All plugin source files parse cleanly.")
