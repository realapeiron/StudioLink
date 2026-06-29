--!strict
-- Pure line/text manipulation for surgical script editing (edit/insert/delete
-- _script_lines). No Roblox API — unit-tested under Lune. ScriptTools reads/
-- writes instance.Source and delegates the actual transform here.

local LineEdit = {}

-- Insert `content` after 1-indexed `afterLine` (0 = before the first line).
function LineEdit.insertAfter(source: string, afterLine: number, content: string): string
	local lines = source:split("\n")
	local insertAt = math.clamp(afterLine, 0, #lines)
	local contentLines = content:split("\n")
	for i = #contentLines, 1, -1 do
		table.insert(lines, insertAt + 1, contentLines[i])
	end
	return table.concat(lines, "\n")
end

-- Delete lines `startLine`..`endLine` inclusive (1-indexed).
function LineEdit.deleteRange(source: string, startLine: number, endLine: number): string
	local lines = source:split("\n")
	local s = math.max(1, startLine)
	local e = math.min(#lines, endLine)
	local out: { string } = {}
	for i, line in ipairs(lines) do
		if i < s or i > e then
			table.insert(out, line)
		end
	end
	return table.concat(out, "\n")
end

-- Replace the first occurrence of `oldText` with `newText`. If `startLine` is
-- given, search begins at that line (to disambiguate). Returns (newSource, nil)
-- on success or (nil, errorMessage) if `oldText` was not found.
function LineEdit.replaceText(
	source: string,
	oldText: string,
	newText: string,
	startLine: number?
): (string?, string?)
	if oldText == "" then
		return nil, "old_string must not be empty"
	end
	local searchFrom = 1
	if startLine and startLine > 1 then
		local lines = source:split("\n")
		local off = 0
		for i = 1, math.min(startLine - 1, #lines) do
			off += #lines[i] + 1 -- +1 for the newline
		end
		searchFrom = off + 1
	end
	local idx = source:find(oldText, searchFrom, true) -- plain text, not a pattern
	if not idx then
		return nil, "old_string not found in script source"
	end
	local before = source:sub(1, idx - 1)
	local after = source:sub(idx + #oldText)
	return before .. newText .. after, nil
end

return LineEdit
