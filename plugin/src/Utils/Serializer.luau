--!strict
-- Serializer: Advanced value serialization for plugin responses

local HttpService = game:GetService("HttpService")

local Serializer = {}

-- Serialize any Luau value to a JSON-safe representation
function Serializer.serialize(value: any): any
	local t = typeof(value)

	if t == "nil" then
		return nil
	elseif t == "boolean" or t == "number" or t == "string" then
		return value
	elseif t == "table" then
		-- Check if array
		local isArray = true
		local maxIndex = 0
		for k, _ in pairs(value) do
			if type(k) ~= "number" or k ~= math.floor(k) or k < 1 then
				isArray = false
				break
			end
			maxIndex = math.max(maxIndex, k)
		end
		isArray = isArray and maxIndex == #value

		if isArray then
			local arr = {}
			for _, v in ipairs(value) do
				table.insert(arr, Serializer.serialize(v))
			end
			return arr
		else
			local obj = {}
			for k, v in pairs(value) do
				obj[tostring(k)] = Serializer.serialize(v)
			end
			return obj
		end
	elseif t == "Instance" then
		return {
			__type = "Instance",
			ClassName = value.ClassName,
			Name = value.Name,
			FullName = value:GetFullName(),
		}
	elseif t == "Vector3" then
		return { __type = "Vector3", X = value.X, Y = value.Y, Z = value.Z }
	elseif t == "CFrame" then
		return { __type = "CFrame", Position = Serializer.serialize(value.Position) }
	elseif t == "Color3" then
		return { __type = "Color3", R = value.R, G = value.G, B = value.B }
	elseif t == "UDim2" then
		return {
			__type = "UDim2",
			X = { Scale = value.X.Scale, Offset = value.X.Offset },
			Y = { Scale = value.Y.Scale, Offset = value.Y.Offset },
		}
	elseif t == "EnumItem" then
		return tostring(value)
	else
		return tostring(value)
	end
end

-- Encode a value to JSON string
function Serializer.toJSON(value: any): string
	local serialized = Serializer.serialize(value)
	local ok, result = pcall(function()
		return HttpService:JSONEncode(serialized)
	end)
	if ok then
		return result
	else
		return HttpService:JSONEncode({ error = "Serialization failed: " .. tostring(result) })
	end
end

return Serializer
