--!strict
-- InstanceTools: Instance management utilities for StudioLink

local Serializer = require(script.Parent.Parent.Utils.Serializer)

local InstanceTools = {}

-- Resolve a dot-separated path like "Workspace.Model.Part" to an Instance
local function resolvePath(path: string): Instance?
	local parts = path:split(".")
	local current: Instance = game

	-- Handle "game" prefix
	local startIndex = 1
	if parts[1] == "game" then
		startIndex = 2
	end

	for i = startIndex, #parts do
		local child = current:FindFirstChild(parts[i])
		if not child then
			-- Try GetService for top-level services
			if current == game then
				local ok, svc = pcall(function()
					return game:GetService(parts[i])
				end)
				if ok and svc then
					child = svc
				end
			end
		end
		if not child then
			return nil
		end
		current = child
	end

	return current
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
		if type(value) == "table" then
			return Vector3.new(value.X or value[1] or 0, value.Y or value[2] or 0, value.Z or value[3] or 0)
		end
	elseif valueType == "Color3" then
		if type(value) == "table" then
			return Color3.new(value.R or value[1] or 0, value.G or value[2] or 0, value.B or value[3] or 0)
		end
	elseif valueType == "UDim2" then
		if type(value) == "table" then
			return UDim2.new(value[1] or 0, value[2] or 0, value[3] or 0, value[4] or 0)
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

	local newVal = deserializeValue(value, valueType)
	local updated = 0
	local failed = 0
	local errors: { string } = {}

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

	-- Set properties if provided
	if properties and type(properties) == "table" then
		for propName, propVal in pairs(properties) do
			pcall(function()
				(instance :: any)[propName] = propVal
			end)
		end
	end

	-- Set name if provided in properties
	if properties and properties.Name then
		instance.Name = tostring(properties.Name)
	end

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

	instance:Destroy()

	return true, {
		deleted = true,
		name = name,
		className = className,
		descendantsRemoved = descendantCount,
	}, nil
end

return InstanceTools
