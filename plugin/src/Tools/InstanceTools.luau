--!strict
-- InstanceTools: Instance management utilities for StudioLink

local Serializer = require(script.Parent.Parent.Utils.Serializer)
local PathResolver = require(script.Parent.Parent.Utils.PathResolver)

local InstanceTools = {}

local function resolvePath(path: string): Instance?
	return PathResolver.resolve(path)
end

-- Try to parse a JSON string into a table
local function tryJsonDecode(value: any): any
	if type(value) == "string" then
		local ok, decoded = pcall(function()
			return game:GetService("HttpService"):JSONDecode(value)
		end)
		if ok and type(decoded) == "table" then
			return decoded
		end
	end
	return value
end

-- Deserialize a value from JSON to the appropriate Roblox type
local function deserializeValue(value: any, valueType: string?): any
	if valueType == nil or valueType == "string" then
		return value
	elseif valueType == "number" then
		return tonumber(value)
	elseif valueType == "boolean" then
		return value == true or value == "true"
	elseif valueType == "Vector3" then
		local v = tryJsonDecode(value)
		if type(v) == "table" then
			return Vector3.new(v.X or v[1] or 0, v.Y or v[2] or 0, v.Z or v[3] or 0)
		elseif type(v) == "string" then
			local parts = v:gsub("[%[%]%(%)%s]", ""):split(",")
			if #parts >= 3 then
				return Vector3.new(tonumber(parts[1]) or 0, tonumber(parts[2]) or 0, tonumber(parts[3]) or 0)
			end
		end
	elseif valueType == "Color3" then
		local v = tryJsonDecode(value)
		if type(v) == "table" then
			return Color3.new(v.R or v[1] or 0, v.G or v[2] or 0, v.B or v[3] or 0)
		elseif type(v) == "string" then
			local parts = v:gsub("[%[%]%(%)%s]", ""):split(",")
			if #parts >= 3 then
				return Color3.new(tonumber(parts[1]) or 0, tonumber(parts[2]) or 0, tonumber(parts[3]) or 0)
			end
		end
	elseif valueType == "UDim2" then
		local v = tryJsonDecode(value)
		if type(v) == "table" then
			return UDim2.new(v[1] or 0, v[2] or 0, v[3] or 0, v[4] or 0)
		elseif type(v) == "string" then
			local parts = v:gsub("[%[%]%(%)%s]", ""):split(",")
			if #parts >= 4 then
				return UDim2.new(tonumber(parts[1]) or 0, tonumber(parts[2]) or 0, tonumber(parts[3]) or 0, tonumber(parts[4]) or 0)
			end
		end
	elseif valueType == "BrickColor" then
		return BrickColor.new(tostring(value))
	elseif valueType == "Enum" then
		-- Try to resolve Enum, e.g. "Enum.Material.Plastic"
		local parts = tostring(value):split(".")
		if #parts == 3 then
			local ok, result = pcall(function()
				return (Enum :: any)[parts[2]][parts[3]]
			end)
			if ok then return result end
		end
	end
	return value
end

-- Build a tree representation of instances
local function buildTree(instance: Instance, depth: number, maxDepth: number): { [string]: any }
	local node: { [string]: any } = {
		name = instance.Name,
		className = instance.ClassName,
	}

	if depth < maxDepth then
		local children = {}
		for _, child in ipairs(instance:GetChildren()) do
			table.insert(children, buildTree(child, depth + 1, maxDepth))
		end
		if #children > 0 then
			node.children = children
			node.childCount = #children
		end
	else
		local count = #instance:GetChildren()
		if count > 0 then
			node.childCount = count
			node.truncated = true
		end
	end

	return node
end

-- Get known properties for a given class
local function getProperties(instance: Instance): { [string]: any }
	local props: { [string]: any } = {
		Name = instance.Name,
		ClassName = instance.ClassName,
		FullName = instance:GetFullName(),
	}

	-- Common properties with pcall safety
	local commonProps = {
		"Parent", "Archivable",
	}

	-- BasePart properties
	if instance:IsA("BasePart") then
		local partProps = {
			"Position", "Size", "CFrame", "Orientation", "Rotation",
			"Anchored", "CanCollide", "CanTouch", "CanQuery",
			"Transparency", "Material", "Color", "BrickColor",
			"Shape", "Massless", "RootPriority",
		}
		for _, p in ipairs(partProps) do table.insert(commonProps, p) end
	end

	-- Model properties
	if instance:IsA("Model") then
		table.insert(commonProps, "PrimaryPart")
		table.insert(commonProps, "WorldPivot")
	end

	-- GUI properties
	if instance:IsA("GuiObject") then
		local guiProps = {
			"Position", "Size", "AnchorPoint", "Visible",
			"BackgroundColor3", "BackgroundTransparency",
			"BorderColor3", "BorderSizePixel", "ZIndex",
			"LayoutOrder", "Active", "ClipsDescendants",
		}
		for _, p in ipairs(guiProps) do table.insert(commonProps, p) end
	end

	-- TextLabel/TextButton/TextBox
	if instance:IsA("TextLabel") or instance:IsA("TextButton") or instance:IsA("TextBox") then
		local textProps = { "Text", "TextColor3", "TextSize", "Font", "TextWrapped", "TextScaled" }
		for _, p in ipairs(textProps) do table.insert(commonProps, p) end
	end

	-- ImageLabel/ImageButton
	if instance:IsA("ImageLabel") or instance:IsA("ImageButton") then
		table.insert(commonProps, "Image")
		table.insert(commonProps, "ImageColor3")
		table.insert(commonProps, "ImageTransparency")
	end

	-- Light properties
	if instance:IsA("Light") then
		local lightProps = { "Brightness", "Color", "Enabled", "Range", "Shadows" }
		for _, p in ipairs(lightProps) do table.insert(commonProps, p) end
	end

	-- Script properties
	if instance:IsA("LuaSourceContainer") then
		table.insert(commonProps, "Enabled")
		table.insert(commonProps, "RunContext")
	end

	-- ValueBase properties
	if instance:IsA("ValueBase") then
		table.insert(commonProps, "Value")
	end

	-- Read properties with pcall
	for _, propName in ipairs(commonProps) do
		local ok, val = pcall(function()
			return (instance :: any)[propName]
		end)
		if ok and val ~= nil then
			props[propName] = Serializer.serialize(val)
		end
	end

	-- Attributes
	local attrs = instance:GetAttributes()
	local hasAttrs = false
	for _ in pairs(attrs) do hasAttrs = true; break end
	if hasAttrs then
		props.Attributes = attrs
	end

	-- Tags
	local tags = instance:GetTags()
	if #tags > 0 then
		props.Tags = tags
	end

	return props
end

-- ═══════════════════════════════════════════
-- TOOL IMPLEMENTATIONS
-- ═══════════════════════════════════════════

function InstanceTools.getFileTree(args: { [string]: any }): (boolean, any, string?)
	local targetPath = args.path or ""
	local maxDepth = args.depth or 10

	local services = {
		game:GetService("Workspace"),
		game:GetService("ServerScriptService"),
		game:GetService("ServerStorage"),
		game:GetService("ReplicatedStorage"),
		game:GetService("ReplicatedFirst"),
		game:GetService("StarterGui"),
		game:GetService("StarterPack"),
		game:GetService("StarterPlayer"),
	}

	if targetPath ~= "" then
		local target = resolvePath(targetPath)
		if not target then
			return false, nil, "Instance not found: " .. targetPath
		end
		return true, buildTree(target, 0, maxDepth), nil
	end

	local tree = {}
	for _, svc in ipairs(services) do
		table.insert(tree, buildTree(svc, 0, maxDepth))
	end

	return true, { services = tree }, nil
end

function InstanceTools.getProperties(args: { [string]: any }): (boolean, any, string?)
	local path = args.path
	if not path or path == "" then
		return false, nil, "Missing required parameter: path"
	end

	local instance = resolvePath(path)
	if not instance then
		return false, nil, "Instance not found: " .. path
	end

	return true, getProperties(instance), nil
end

function InstanceTools.setProperty(args: { [string]: any }): (boolean, any, string?)
	local path = args.path
	local propName = args.property
	local value = args.value
	local valueType = args.valueType

	if not path or not propName then
		return false, nil, "Missing required parameters: path, property"
	end

	local instance = resolvePath(path)
	if not instance then
		return false, nil, "Instance not found: " .. path
	end

	local oldValue = nil
	pcall(function() oldValue = (instance :: any)[propName] end)

	-- Record waypoint BEFORE change (so undo reverts to pre-change state)
	pcall(function()
		local CHS = game:GetService("ChangeHistoryService")
		;(CHS :: any):SetWaypoint("StudioLink: Set " .. propName .. " on " .. instance.Name)
	end)

	local newVal = deserializeValue(value, valueType)
	local ok, err = pcall(function()
		(instance :: any)[propName] = newVal
	end)

	if not ok then
		return false, nil, "Failed to set property: " .. tostring(err)
	end

	return true, {
		path = path,
		property = propName,
		oldValue = Serializer.serialize(oldValue),
		newValue = Serializer.serialize((instance :: any)[propName]),
	}, nil
end

function InstanceTools.massSetProperty(args: { [string]: any }): (boolean, any, string?)
	local paths = args.paths
	local propName = args.property
	local value = args.value
	local valueType = args.valueType

	if not paths or not propName then
		return false, nil, "Missing required parameters: paths, property"
	end
	if type(paths) ~= "table" then
		return false, nil, "paths must be an array of instance paths"
	end

	local newVal = deserializeValue(value, valueType)
	local updated = 0
	local failed = 0
	local errors: { string } = {}

	-- Record waypoint BEFORE changes (so undo reverts to pre-change state)
	pcall(function()
		local CHS = game:GetService("ChangeHistoryService")
		;(CHS :: any):SetWaypoint("StudioLink: Mass set " .. propName)
	end)

	for _, path in ipairs(paths) do
		local instance = resolvePath(path)
		if instance then
			local ok, err = pcall(function()
				(instance :: any)[propName] = newVal
			end)
			if ok then
				updated += 1
			else
				failed += 1
				table.insert(errors, path .. ": " .. tostring(err))
			end
		else
			failed += 1
			table.insert(errors, path .. ": not found")
		end
	end

	return true, {
		updated = updated,
		failed = failed,
		errors = if #errors > 0 then errors else nil,
	}, nil
end

function InstanceTools.createInstance(args: { [string]: any }): (boolean, any, string?)
	local className = args.className
	local parentPath = args.parentPath or args.parent
	local properties = args.properties

	if not className then
		return false, nil, "Missing required parameter: className"
	end

	local parent: Instance = game:GetService("Workspace")
	if parentPath and parentPath ~= "" then
		local resolved = resolvePath(parentPath)
		if not resolved then
			return false, nil, "Parent not found: " .. parentPath
		end
		parent = resolved
	end

	local ok, instance = pcall(function()
		return Instance.new(className)
	end)

	if not ok or not instance then
		return false, nil, "Failed to create instance: " .. tostring(instance)
	end

	-- Set properties if provided. Infer each property's type from the freshly
	-- created instance and route through deserializeValue, so JSON-origin values
	-- ({x,y,z}, "true", "Enum.Material.Plastic", …) are coerced the same way
	-- set_property does — instead of being assigned raw and silently failing.
	if properties and type(properties) == "table" then
		for propName, propVal in pairs(properties) do
			pcall(function()
				local currentType = typeof((instance :: any)[propName])
				if currentType == "EnumItem" then
					currentType = "Enum"
				end
				;(instance :: any)[propName] = deserializeValue(propVal, currentType)
			end)
		end
	end

	-- Record waypoint BEFORE parenting (so undo reverts to pre-creation state)
	pcall(function()
		local CHS = game:GetService("ChangeHistoryService")
		;(CHS :: any):SetWaypoint("StudioLink: Create " .. className .. " in " .. parent.Name)
	end)

	instance.Parent = parent

	return true, {
		fullName = instance:GetFullName(),
		className = instance.ClassName,
		name = instance.Name,
		created = true,
	}, nil
end

function InstanceTools.deleteInstance(args: { [string]: any }): (boolean, any, string?)
	local path = args.path
	if not path or path == "" then
		return false, nil, "Missing required parameter: path"
	end

	local instance = resolvePath(path)
	if not instance then
		return false, nil, "Instance not found: " .. path
	end

	local name = instance.Name
	local className = instance.ClassName
	local descendantCount = #instance:GetDescendants()

	-- Record waypoint BEFORE destruction (so undo can restore the instance)
	pcall(function()
		local CHS = game:GetService("ChangeHistoryService")
		;(CHS :: any):SetWaypoint("StudioLink: Delete " .. name)
	end)

	instance:Destroy()

	return true, {
		deleted = true,
		name = name,
		className = className,
		descendantsRemoved = descendantCount,
	}, nil
end

-- ───────────────────────── Attributes & Tags (v0.8.0) ─────────────────────────

local CollectionService = game:GetService("CollectionService")

function InstanceTools.getAttributes(args: { [string]: any }): (boolean, any, string?)
	local path = args.path
	if type(path) ~= "string" or path == "" then
		return false, nil, "path is required"
	end
	local instance = resolvePath(path)
	if not instance then
		return false, nil, "Instance not found: " .. path
	end
	local out: { [string]: any } = {}
	for k, v in pairs(instance:GetAttributes()) do
		out[k] = { value = tostring(v), type = typeof(v) }
	end
	return true, { path = path, attributes = out }, nil
end

function InstanceTools.setAttribute(args: { [string]: any }): (boolean, any, string?)
	local path, name = args.path, args.name
	if type(path) ~= "string" or path == "" then
		return false, nil, "path is required"
	end
	if type(name) ~= "string" or name == "" then
		return false, nil, "name is required"
	end
	local instance = resolvePath(path)
	if not instance then
		return false, nil, "Instance not found: " .. path
	end
	local val = deserializeValue(args.value, args.valueType)
	local ok, err = pcall(function()
		instance:SetAttribute(name, val)
	end)
	if not ok then
		return false, nil, "SetAttribute failed: " .. tostring(err)
	end
	return true, { path = path, name = name, set = true }, nil
end

function InstanceTools.deleteAttribute(args: { [string]: any }): (boolean, any, string?)
	local path, name = args.path, args.name
	if type(path) ~= "string" or path == "" then
		return false, nil, "path is required"
	end
	if type(name) ~= "string" or name == "" then
		return false, nil, "name is required"
	end
	local instance = resolvePath(path)
	if not instance then
		return false, nil, "Instance not found: " .. path
	end
	local ok, err = pcall(function()
		instance:SetAttribute(name, nil)
	end)
	if not ok then
		return false, nil, "delete attribute failed: " .. tostring(err)
	end
	return true, { path = path, name = name, deleted = true }, nil
end

function InstanceTools.bulkSetAttributes(args: { [string]: any }): (boolean, any, string?)
	local path = args.path
	local attributes = args.attributes
	if type(path) ~= "string" or path == "" then
		return false, nil, "path is required"
	end
	if type(attributes) ~= "table" then
		return false, nil, "attributes must be a table of name -> value"
	end
	local instance = resolvePath(path)
	if not instance then
		return false, nil, "Instance not found: " .. path
	end
	local set = 0
	local errors: { string } = {}
	for name, raw in pairs(attributes) do
		-- raw may be a plain value, or { value = X, type = "Vector3" }
		local value, vtype = raw, nil
		if type(raw) == "table" and (raw :: any).value ~= nil then
			value, vtype = (raw :: any).value, (raw :: any).type
		end
		local ok = pcall(function()
			instance:SetAttribute(name, deserializeValue(value, vtype))
		end)
		if ok then
			set += 1
		else
			table.insert(errors, name)
		end
	end
	return true, { path = path, set = set, failed = if #errors > 0 then errors else nil }, nil
end

function InstanceTools.getTags(args: { [string]: any }): (boolean, any, string?)
	local path = args.path
	if type(path) ~= "string" or path == "" then
		return false, nil, "path is required"
	end
	local instance = resolvePath(path)
	if not instance then
		return false, nil, "Instance not found: " .. path
	end
	return true, { path = path, tags = CollectionService:GetTags(instance) }, nil
end

function InstanceTools.addTag(args: { [string]: any }): (boolean, any, string?)
	local path, tag = args.path, args.tag
	if type(path) ~= "string" or path == "" then
		return false, nil, "path is required"
	end
	if type(tag) ~= "string" or tag == "" then
		return false, nil, "tag is required"
	end
	local instance = resolvePath(path)
	if not instance then
		return false, nil, "Instance not found: " .. path
	end
	local ok, err = pcall(function()
		CollectionService:AddTag(instance, tag)
	end)
	if not ok then
		return false, nil, "AddTag failed: " .. tostring(err)
	end
	return true, { path = path, tag = tag, added = true }, nil
end

function InstanceTools.removeTag(args: { [string]: any }): (boolean, any, string?)
	local path, tag = args.path, args.tag
	if type(path) ~= "string" or path == "" then
		return false, nil, "path is required"
	end
	if type(tag) ~= "string" or tag == "" then
		return false, nil, "tag is required"
	end
	local instance = resolvePath(path)
	if not instance then
		return false, nil, "Instance not found: " .. path
	end
	local ok, err = pcall(function()
		CollectionService:RemoveTag(instance, tag)
	end)
	if not ok then
		return false, nil, "RemoveTag failed: " .. tostring(err)
	end
	return true, { path = path, tag = tag, removed = true }, nil
end

function InstanceTools.getTagged(args: { [string]: any }): (boolean, any, string?)
	local tag = args.tag
	if type(tag) ~= "string" or tag == "" then
		return false, nil, "tag is required"
	end
	local out: { string } = {}
	for _, inst in ipairs(CollectionService:GetTagged(tag)) do
		table.insert(out, inst:GetFullName())
	end
	return true, { tag = tag, tagged = out, count = #out }, nil
end

-- ───────────────────────── Discovery (v0.8.0) ─────────────────────────

local DISCOVERY_SERVICES = {
	"Workspace",
	"Players",
	"ReplicatedStorage",
	"ReplicatedFirst",
	"ServerScriptService",
	"ServerStorage",
	"StarterGui",
	"StarterPack",
	"StarterPlayer",
	"Lighting",
	"SoundService",
	"Teams",
}

function InstanceTools.inspectInstance(args: { [string]: any }): (boolean, any, string?)
	local path = args.path
	if type(path) ~= "string" or path == "" then
		return false, nil, "path is required"
	end
	local instance = resolvePath(path)
	if not instance then
		return false, nil, "Instance not found: " .. path
	end
	local attrs: { [string]: any } = {}
	for k, v in pairs(instance:GetAttributes()) do
		attrs[k] = { value = tostring(v), type = typeof(v) }
	end
	local children: { any } = {}
	for _, c in ipairs(instance:GetChildren()) do
		table.insert(children, { name = c.Name, className = c.ClassName })
	end
	return true,
		{
			path = path,
			name = instance.Name,
			className = instance.ClassName,
			fullName = instance:GetFullName(),
			attributes = attrs,
			tags = CollectionService:GetTags(instance),
			childCount = #children,
			children = children,
		},
		nil
end

function InstanceTools.getDescendants(args: { [string]: any }): (boolean, any, string?)
	local path = args.path
	if type(path) ~= "string" or path == "" then
		return false, nil, "path is required"
	end
	local maxDepth = if type(args.maxDepth) == "number" then args.maxDepth else 10
	local classFilter = if type(args.classFilter) == "string" then args.classFilter else nil
	local instance = resolvePath(path)
	if not instance then
		return false, nil, "Instance not found: " .. path
	end
	local results: { any } = {}
	local function walk(inst: Instance, depth: number)
		if depth > maxDepth or #results >= 1000 then
			return
		end
		for _, c in ipairs(inst:GetChildren()) do
			if classFilter == nil or c:IsA(classFilter) then
				table.insert(results, {
					path = c:GetFullName(),
					name = c.Name,
					className = c.ClassName,
					depth = depth,
				})
			end
			walk(c, depth + 1)
		end
	end
	walk(instance, 1)
	return true, { path = path, count = #results, descendants = results }, nil
end

function InstanceTools.getSelection(_args: { [string]: any }): (boolean, any, string?)
	local ok, selection = pcall(function()
		return game:GetService("Selection"):Get()
	end)
	if not ok then
		return false, nil, "could not read Studio selection"
	end
	local out: { any } = {}
	for _, inst in ipairs(selection) do
		table.insert(out, { path = inst:GetFullName(), name = inst.Name, className = inst.ClassName })
	end
	return true, { count = #out, selection = out }, nil
end

function InstanceTools.searchByProperty(args: { [string]: any }): (boolean, any, string?)
	local propName = args.propertyName
	if type(propName) ~= "string" or propName == "" then
		return false, nil, "propertyName is required"
	end
	local target = tostring(args.propertyValue)
	local results: { any } = {}
	for _, svcName in ipairs(DISCOVERY_SERVICES) do
		local svcOk, svc = pcall(function()
			return game:GetService(svcName :: any)
		end)
		if svcOk and svc then
			for _, desc in ipairs(svc:GetDescendants()) do
				if #results >= 200 then
					break
				end
				local readOk, val = pcall(function()
					return (desc :: any)[propName]
				end)
				if readOk and tostring(val) == target then
					table.insert(results, { path = desc:GetFullName(), className = desc.ClassName })
				end
			end
		end
	end
	return true,
		{ propertyName = propName, value = target, count = #results, truncated = #results >= 200, results = results },
		nil
end

function InstanceTools.getServices(_args: { [string]: any }): (boolean, any, string?)
	local out: { any } = {}
	for _, name in ipairs(DISCOVERY_SERVICES) do
		local ok, svc = pcall(function()
			return game:GetService(name :: any)
		end)
		if ok and svc then
			table.insert(out, { name = name, className = svc.ClassName, childCount = #svc:GetChildren() })
		end
	end
	return true, { services = out }, nil
end

function InstanceTools.getPlaceInfo(_args: { [string]: any }): (boolean, any, string?)
	return true,
		{
			placeId = game.PlaceId,
			gameId = game.GameId,
			name = game.Name,
			creatorId = game.CreatorId,
			creatorType = tostring(game.CreatorType),
			placeVersion = game.PlaceVersion,
		},
		nil
end

function InstanceTools.getClassInfo(args: { [string]: any }): (boolean, any, string?)
	local className = args.className
	if type(className) ~= "string" or className == "" then
		return false, nil, "className is required"
	end
	local ok, instance = pcall(function()
		return Instance.new(className :: any)
	end)
	if not ok or not instance then
		return false, nil, "invalid or non-instantiable class: " .. className
	end
	local isA: { string } = {}
	for _, base in
		ipairs({
			"Instance",
			"BasePart",
			"GuiObject",
			"GuiBase2d",
			"LuaSourceContainer",
			"ValueBase",
			"Constraint",
			"Light",
			"PVInstance",
		})
	do
		if instance:IsA(base) then
			table.insert(isA, base)
		end
	end
	instance:Destroy()
	return true, { className = className, valid = true, isA = isA }, nil
end

return InstanceTools
