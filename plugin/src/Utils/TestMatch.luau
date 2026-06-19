--!strict
-- Pure name-matching for test_run, testable under Lune. The old inline logic
-- also matched any name CONTAINING "test", which pulled in unrelated modules
-- (LatestConfig, ContestManager, TestUtilities) and ran their top-level side
-- effects. Convention is the .spec / .test suffix; test cases start with "test".

local TestMatch = {}

-- A script is a test suite if its name ends with .spec or .test.
function TestMatch.isTestScriptName(name: string): boolean
	local lower = name:lower()
	return lower:find("%.spec$") ~= nil or lower:find("%.test$") ~= nil
end

-- A module member is a test case if its name starts with "test".
function TestMatch.isTestFunctionName(name: string): boolean
	return name:lower():find("^test") ~= nil
end

return TestMatch
