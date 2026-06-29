--!strict
-- Lune test for find_and_replace_in_scripts core logic.
-- Run: lune run plugin/tests/FindReplace.test.luau

local FR = require("../src/Utils/FindReplace")

local failures = 0
local function check(name: string, actual: any, expected: any)
	if actual == expected then
		print("  ok   " .. name)
	else
		failures += 1
		print("  FAIL " .. name .. " — got " .. tostring(actual) .. ", expected " .. tostring(expected))
	end
end

local function rep(s, p, r, up): string
	local out = FR.replaceInSource(s, p, r, up)
	return out
end
local function cnt(s, p, r, up): number
	local _, n = FR.replaceInSource(s, p, r, up)
	return n
end

print("literal mode")
check("replace all occurrences", rep("foo bar foo", "foo", "X", false), "X bar X")
check("count", cnt("foo bar foo", "foo", "X", false), 2)
check("special chars are literal", rep("a.b.c", ".", "-", false), "a-b-c")
check("no match -> unchanged", rep("abc", "xyz", "Y", false), "abc")
check("no match -> count 0", cnt("abc", "xyz", "Y", false), 0)
check("percent in replacement is literal", rep("give 50 now", "50", "100%", false), "give 100% now")

print("pattern mode")
check("digit class", rep("a1b2c3", "%d", "#", true), "a#b#c#")
check("pattern count", cnt("a1b2c3", "%d", "#", true), 3)

if failures > 0 then
	error(tostring(failures) .. " test(s) failed")
end
print("\nAll FindReplace tests passed.")
