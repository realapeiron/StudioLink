--!strict
-- Lune test for surgical script-editing line logic.
-- Run: lune run plugin/tests/LineEdit.test.luau

local LineEdit = require("../src/Utils/LineEdit")

local failures = 0
local function check(name: string, actual: any, expected: any)
	if actual == expected then
		print("  ok   " .. name)
	else
		failures += 1
		print("  FAIL " .. name .. " — got " .. tostring(actual) .. ", expected " .. tostring(expected))
	end
end

print("insertAfter")
check("at start (0)", LineEdit.insertAfter("a\nb", 0, "X"), "X\na\nb")
check("after line 1", LineEdit.insertAfter("a\nb", 1, "X"), "a\nX\nb")
check("after last line", LineEdit.insertAfter("a\nb", 2, "X"), "a\nb\nX")
check("multiline content", LineEdit.insertAfter("a\nb", 1, "X\nY"), "a\nX\nY\nb")
check("clamps beyond end", LineEdit.insertAfter("a", 9, "Z"), "a\nZ")

print("deleteRange (1-indexed inclusive)")
check("first line", LineEdit.deleteRange("a\nb\nc", 1, 1), "b\nc")
check("last two", LineEdit.deleteRange("a\nb\nc", 2, 3), "a")
check("middle", LineEdit.deleteRange("a\nb\nc\nd", 2, 3), "a\nd")
check("all", LineEdit.deleteRange("a\nb", 1, 2), "")

print("replaceText")
local r1 = LineEdit.replaceText("foo\nbar\nbaz", "bar", "BAR")
check("replace found", r1, "foo\nBAR\nbaz")
local r2 = LineEdit.replaceText("x = 1\ny = 1", "= 1", "= 2")
check("replace first only", r2, "x = 2\ny = 1")
local r3, e3 = LineEdit.replaceText("foo\nbar", "xyz", "X")
check("not found -> nil", r3, nil)
check("not found -> error msg present", type(e3) == "string" and #e3 > 0, true)

if failures > 0 then
	error(tostring(failures) .. " test(s) failed")
end
print("\nAll LineEdit tests passed.")
