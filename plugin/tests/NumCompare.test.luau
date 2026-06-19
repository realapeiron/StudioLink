--!strict
-- Lune test for the pure comparison logic behind wait_for_condition.
-- Run with: lune run plugin/tests/NumCompare.test.luau
-- (No Roblox API touched, so it runs standalone.)

local NumCompare = require("../src/Utils/NumCompare")

local failures = 0
local function check(name: string, actual: any, expected: any)
	if actual == expected then
		print("  ok   " .. name)
	else
		failures += 1
		print("  FAIL " .. name .. " — got " .. tostring(actual) .. ", expected " .. tostring(expected))
	end
end

print("NumCompare.compare")

-- Behavior that must be preserved (v0.7.2 ordering coercion + plain equality)
check("ordering: numeric string > number", NumCompare.compare("100", ">", 50), true)
check("ordering: number < numeric string", NumCompare.compare(50, "<", "100"), true)
check("equality: number == number", NumCompare.compare(5, "==", 5), true)
check("equality: string == string", NumCompare.compare("abc", "==", "abc"), true)
check("inequality: string != string", NumCompare.compare("abc", "!=", "def"), true)
check("ordering: non-numeric returns false", NumCompare.compare("abc", ">", 5), false)

-- The v0.7.3 fix: == / != must coerce when BOTH sides are numeric.
-- A Roblox numeric-string property vs a JSON number is the common case.
check("equality: number == numeric string", NumCompare.compare(100, "==", "100"), true)
check("equality: numeric string == number", NumCompare.compare("100", "==", 100), true)
check("inequality: number != numeric string (same value)", NumCompare.compare(100, "!=", "100"), false)
check("equality: numeric strings differ", NumCompare.compare("100", "==", "200"), false)

-- Guard: a real string target must NOT be coerced away
check("equality: non-numeric strings still compare raw", NumCompare.compare("on", "==", "on"), true)

print("isValidOperator")
check("valid ==", NumCompare.isValidOperator("=="), true)
check("invalid ===", NumCompare.isValidOperator("==="), false)

if failures > 0 then
	error(tostring(failures) .. " test(s) failed")
end
print("\nAll NumCompare tests passed.")
