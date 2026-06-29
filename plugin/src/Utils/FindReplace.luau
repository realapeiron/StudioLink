--!strict
-- Pure find/replace over a single source string, for find_and_replace_in_scripts.
-- Literal mode escapes Lua pattern magic in both needle and replacement; pattern
-- mode passes them through (so %d, captures %1, etc. work). Unit-tested in Lune.

local FindReplace = {}

local function escapePattern(s: string): string
	return (s:gsub("[%(%)%.%%%+%-%*%?%[%]%^%$]", "%%%1"))
end

-- Returns (newSource, replacementCount).
function FindReplace.replaceInSource(
	source: string,
	pattern: string,
	replacement: string,
	usePattern: boolean
): (string, number)
	local searchPattern = if usePattern then pattern else escapePattern(pattern)
	-- In literal mode the replacement is verbatim, so escape gsub's own `%`.
	local replaceWith = if usePattern then replacement else (replacement:gsub("%%", "%%%%"))
	local result, count = source:gsub(searchPattern, replaceWith)
	return result, count
end

return FindReplace
