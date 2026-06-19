--!strict
-- Lune test for test_run's name matching.
-- Run: lune run plugin/tests/TestMatch.test.luau

local TM = require("../src/Utils/TestMatch")

local failures = 0
local function check(name: string, actual: any, expected: any)
	if actual == expected then
		print("  ok   " .. name)
	else
		failures += 1
		print("  FAIL " .. name .. " — got " .. tostring(actual) .. ", expected " .. tostring(expected))
	end
end

print("isTestScriptName (.spec / .test suffix only)")
check(".spec suffix", TM.isTestScriptName("PlayerMovement.spec"), true)
check(".test suffix", TM.isTestScriptName("Combat.test"), true)
check("case-insensitive suffix", TM.isTestScriptName("Combat.Spec"), true)
-- These contain "test" but are NOT test suites — must not match:
check("LatestConfig is not a test", TM.isTestScriptName("LatestConfig"), false)
check("ContestManager is not a test", TM.isTestScriptName("ContestManager"), false)
check("TestUtilities (helper) is not a suite", TM.isTestScriptName("TestUtilities"), false)
check("unrelated name", TM.isTestScriptName("PlayerData"), false)

print("isTestFunctionName (test* prefix)")
check("testJump", TM.isTestFunctionName("testJump"), true)
check("test_jump", TM.isTestFunctionName("test_jump"), true)
check("case-insensitive prefix", TM.isTestFunctionName("TestJump"), true)
-- "latest" contains "test" but is not a test fn:
check("latest is not a test fn", TM.isTestFunctionName("latest"), false)
check("setup is not a test fn", TM.isTestFunctionName("setup"), false)

if failures > 0 then
	error(tostring(failures) .. " test(s) failed")
end
print("\nAll TestMatch tests passed.")
