--!strict
-- Lune test for the viewport_capture base64 encoder (RFC 4648 vectors).
-- Run: lune run plugin/tests/Base64.test.luau

local Base64 = require("../src/Utils/Base64")

local failures = 0
local function check(name: string, actual: any, expected: any)
	if actual == expected then
		print("  ok   " .. name)
	else
		failures += 1
		print("  FAIL " .. name .. " — got " .. tostring(actual) .. ", expected " .. tostring(expected))
	end
end

print("Base64.encodeString (RFC 4648 §10 vectors)")
check("empty", Base64.encodeString(""), "")
check("f", Base64.encodeString("f"), "Zg==")
check("fo", Base64.encodeString("fo"), "Zm8=")
check("foo", Base64.encodeString("foo"), "Zm9v")
check("foob", Base64.encodeString("foob"), "Zm9vYg==")
check("fooba", Base64.encodeString("fooba"), "Zm9vYmE=")
check("foobar", Base64.encodeString("foobar"), "Zm9vYmFy")
check("Man", Base64.encodeString("Man"), "TWFu")
check("hello world", Base64.encodeString("hello world"), "aGVsbG8gd29ybGQ=")

print("binary bytes (RGBA-like)")
-- bytes 255,0,128,255 → base64
local buf = buffer.create(4)
buffer.writeu8(buf, 0, 255)
buffer.writeu8(buf, 1, 0)
buffer.writeu8(buf, 2, 128)
buffer.writeu8(buf, 3, 255)
check("ff0080ff", Base64.encode(buf), "/wCA/w==")

if failures > 0 then
	error(tostring(failures) .. " test(s) failed")
end
print("\nAll Base64 tests passed.")
