--!strict
-- Lune test for the security_scan remote-validation heuristics.
-- Run with: lune run plugin/tests/RemoteValidation.test.luau

local RV = require("../src/Utils/RemoteValidation")

local failures = 0
local function check(name: string, actual: any, expected: any)
	if actual == expected then
		print("  ok   " .. name)
	else
		failures += 1
		print("  FAIL " .. name .. " — got " .. tostring(actual) .. ", expected " .. tostring(expected))
	end
end

print("mentions (plain-text, not pattern)")
check("literal match", RV.mentions("local r = game.X.MyRemote", "MyRemote"), true)
check("no match", RV.mentions("nothing here", "MyRemote"), false)
-- '.' must be literal, not a wildcard: "Data.Save" must NOT match "DataXSave"
check("dot is literal not wildcard", RV.mentions("local DataXSave = 1", "Data.Save"), false)
-- magic chars in a remote name must not error or mis-match
check("parens are literal", RV.mentions("BuyItem()", "Buy(Item)"), false)
check("dash is literal exact", RV.mentions("fire Buy-Item now", "Buy-Item"), true)

print("hasValidation (strong guards only)")
check("typeof counts", RV.hasValidation("if typeof(x) == 'number' then end"), true)
check("type counts", RV.hasValidation("assert(type(n) == 'number')"), true)
check("assert counts", RV.hasValidation("assert(player)"), true)
-- A bare if-then is NOT evidence of input validation.
check("bare if-then is not validation", RV.hasValidation("if player.Health > 0 then heal() end"), false)
check("no guard at all", RV.hasValidation("doThing(a, b)"), false)

print("validatesRemote (mentions AND guard)")
check(
	"typeof guard near remote -> validated",
	RV.validatesRemote({ "MyRemote.OnServerEvent:Connect(function(p, n) assert(typeof(n) == 'number') end)" }, "MyRemote"),
	true
)
check(
	"only if-then near remote -> NOT validated",
	RV.validatesRemote({ "MyRemote.OnServerEvent:Connect(function(p) if p then give(p) end end)" }, "MyRemote"),
	false
)
check(
	"pattern-injected name does not falsely validate via unrelated script",
	RV.validatesRemote({ "local DataXSave = 1; assert(true)" }, "Data.Save"),
	false
)
check("no script mentions remote -> not validated", RV.validatesRemote({ "unrelated", "also unrelated" }, "MyRemote"), false)

if failures > 0 then
	error(tostring(failures) .. " test(s) failed")
end
print("\nAll RemoteValidation tests passed.")
