--!strict
-- TreeWalker: Recursive instance traversal utility

local TreeWalker = {}

-- Walk all descendants and call callback for each
function TreeWalker.walkDescendants(root: Instance, callback: (Instance) -> ())
	for _, child in ipairs(root:GetChildren()) do
		callback(child)
		TreeWalker.walkDescendants(child, callback)
	end
end

-- Walk all descendants matching a class filter
function TreeWalker.walkByClass(root: Instance, className: string, callback: (Instance) -> ())
	TreeWalker.walkDescendants(root, function(instance)
		if instance:IsA(className) then
			callback(instance)
		end
	end)
end

-- Collect all scripts (Script, LocalScript, ModuleScript)
function TreeWalker.collectScripts(root: Instance?): { Instance }
	local scripts = {}
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

	local searchRoot = if root then { root } else services

	for _, svc in ipairs(searchRoot) do
		TreeWalker.walkDescendants(svc, function(instance)
			if instance:IsA("LuaSourceContainer") then
				table.insert(scripts, instance)
			end
		end)
	end

	return scripts
end

-- Collect all instances of a given class across main services
function TreeWalker.collectByClass(className: string): { Instance }
	local results = {}
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

	for _, svc in ipairs(services) do
		TreeWalker.walkByClass(svc, className, function(instance)
			table.insert(results, instance)
		end)
	end

	return results
end

-- Serialize an instance tree to a table (for snapshots)
function TreeWalker.serializeTree(instance: Instance, depth: number?): { [string]: any }
	local maxDepth = depth or 50
	if maxDepth <= 0 then return {} end

	local data: { [string]: any } = {
		Name = instance.Name,
		ClassName = instance.ClassName,
		FullName = instance:GetFullName(),
	}

	-- Serialize properties for BaseParts
	if instance:IsA("BasePart") then
		data.Position = tostring(instance.Position)
		data.Size = tostring(instance.Size)
		data.Material = tostring(instance.Material)
		data.Color = tostring(instance.Color)
		data.Transparency = (instance :: BasePart).Transparency
		data.Anchored = (instance :: BasePart).Anchored
	end

	-- Serialize script source
	if instance:IsA("LuaSourceContainer") then
		local ok, src = pcall(function() return (instance :: any).Source end)
		if ok then
			data.Source = src
		end
	end

	-- Serialize attributes
	local attrs = instance:GetAttributes()
	local hasAttrs = false
	for _ in pairs(attrs) do hasAttrs = true; break end
	if hasAttrs then
		data.Attributes = attrs
	end

	-- Serialize tags
	local tags = instance:GetTags()
	if #tags > 0 then
		data.Tags = tags
	end

	-- Recurse children
	local children = {}
	for _, child in ipairs(instance:GetChildren()) do
		table.insert(children, TreeWalker.serializeTree(child, maxDepth - 1))
	end
	if #children > 0 then
		data.Children = children
	end

	return data
end

return TreeWalker
