--!strict
-- Pure comparison logic shared by wait_for_condition (and testable in isolation
-- via Lune, since it touches no Roblox API). Roblox property reads frequently
-- surface numeric strings (StringValue.Value, attribute strings, IntValue read
-- back through a variant), while tool args arrive from JSON — so a number on one
-- side and a numeric string on the other is the common case. All operators,
-- including == / !=, coerce to numbers when BOTH sides are numeric; otherwise
-- they fall back to raw value equality so non-numeric strings still compare.

local NumCompare = {}

-- Returns the number form of x if it is a number or a numeric string, else nil.
function NumCompare.asNumber(x: any): number?
	if type(x) == "number" then
		return x
	elseif type(x) == "string" then
		return tonumber(x)
	end
	return nil
end

local VALID: { [string]: boolean } = {
	["=="] = true,
	["!="] = true,
	[">"] = true,
	[">="] = true,
	["<"] = true,
	["<="] = true,
}

function NumCompare.isValidOperator(op: string): boolean
	return VALID[op] == true
end

function NumCompare.compare(value: any, op: string, target: any): boolean
	local n1 = NumCompare.asNumber(value)
	local n2 = NumCompare.asNumber(target)
	local bothNumeric = n1 ~= nil and n2 ~= nil

	if op == "==" then
		if bothNumeric then
			return (n1 :: number) == (n2 :: number)
		end
		return value == target
	elseif op == "!=" then
		if bothNumeric then
			return (n1 :: number) ~= (n2 :: number)
		end
		return value ~= target
	end

	-- Ordering operators require both sides to be numeric.
	if not bothNumeric then
		return false
	end
	local a = n1 :: number
	local b = n2 :: number
	if op == ">" then
		return a > b
	elseif op == ">=" then
		return a >= b
	elseif op == "<" then
		return a < b
	elseif op == "<=" then
		return a <= b
	end
	return false
end

return NumCompare
