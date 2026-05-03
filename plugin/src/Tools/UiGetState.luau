--!strict
local PlayHelpers = require(script.Parent.Parent.Utils.PlayHelpers)

local DEFAULT_PROPS = {
	"Text",
	"Visible",
	"AbsolutePosition",
	"AbsoluteSize",
	"Position",
	"Size",
}

local function serializeValue(val: any): any
	local t = typeof(val)
	if t == "Vector2" then
		return { val.X, val.Y }
	elseif t == "Vector3" then
		return { val.X, val.Y, val.Z }
	elseif t == "UDim2" then
		return { val.X.Scale, val.X.Offset, val.Y.Scale, val.Y.Offset }
	elseif t == "UDim" then
		return { val.Scale, val.Offset }
	elseif t == "Color3" then
		return { val.R, val.G, val.B }
	elseif t == "EnumItem" then
		return tostring(val)
	elseif t == "Instance" then
		return val:GetFullName()
	end
	return val
end

return function(args: { [string]: any }): (boolean, any, string?)
	local ctxOk, ctxErr = PlayHelpers.requireContext("play")
	if not ctxOk then
		return false, nil, ctxErr
	end

	local player, perr = PlayHelpers.resolvePlayer(args.player)
	if not player then
		return false, nil, perr
	end

	local instance, serr = PlayHelpers.resolveSelector(args.selector, player)
	if not instance then
		return false, nil, serr
	end

	local props = args.properties or DEFAULT_PROPS
	if type(props) ~= "table" then
		return false, nil, "properties must be an array of strings"
	end

	local result: { [string]: any } = {}
	local missing: { string } = {}
	for _, prop in ipairs(props) do
		local ok, val = pcall(function()
			return (instance :: any)[prop]
		end)
		if ok then
			result[prop] = serializeValue(val)
		else
			table.insert(missing, prop)
		end
	end

	return true,
		{
			target_path = instance:GetFullName(),
			class_name = instance.ClassName,
			properties = result,
			missing_properties = if #missing > 0 then missing else nil,
		},
		nil
end
