--!strict
-- DependencyMap: Map require() chains and detect issues

local TreeWalker = require(script.Parent.Parent.Utils.TreeWalker)

return function(args: { [string]: any }): (boolean, any, string?)
	local modules: { [string]: { path: string, requires: { string }, requiredBy: { string } } } = {}
	local scripts = TreeWalker.collectScripts()

	-- First pass: collect all ModuleScripts
	for _, scriptInstance in ipairs(scripts) do
		if scriptInstance:IsA("ModuleScript") then
			modules[scriptInstance:GetFullName()] = {
				path = scriptInstance:GetFullName(),
				requires = {},
				requiredBy = {},
			}
		end
	end

	-- Second pass: find require() calls
	for _, scriptInstance in ipairs(scripts) do
		local ok, source = pcall(function() return (scriptInstance :: any).Source end)
		if not ok or not source then continue end

		local callerPath = scriptInstance:GetFullName()

		-- Find all require() patterns
		for requireArg in source:gmatch("require%((.-)%)") do
			-- Try to resolve the require target
			local targetName = requireArg:match("%.(%w+)%s*$") or requireArg:match("(%w+)%s*$")
			if targetName then
				-- Find matching module
				for modulePath, moduleData in pairs(modules) do
					if modulePath:match("%." .. targetName .. "$") or modulePath:match("%." .. targetName .. "%.") then
						table.insert(moduleData.requiredBy, callerPath)
						-- Add to caller's requires if it's tracked
						if modules[callerPath] then
							table.insert(modules[callerPath].requires, modulePath)
						end
						break
					end
				end
			end
		end
	end

	-- Detect circular dependencies
	local circular: { { string } } = {}
	local visited: { [string]: boolean } = {}
	local stack: { [string]: boolean } = {}

	local function detectCycles(path: string, chain: { string })
		if stack[path] then
			-- Found a cycle
			local cycleStart = false
			local cycle = {}
			for _, p in ipairs(chain) do
				if p == path then cycleStart = true end
				if cycleStart then table.insert(cycle, p) end
			end
			table.insert(cycle, path)
			table.insert(circular, cycle)
			return
		end
		if visited[path] then return end

		visited[path] = true
		stack[path] = true
		table.insert(chain, path)

		if modules[path] then
			for _, req in ipairs(modules[path].requires) do
				detectCycles(req, chain)
			end
		end

		table.remove(chain)
		stack[path] = false
	end

	for path in pairs(modules) do
		detectCycles(path, {})
	end

	-- Find dead code (modules never required)
	local deadModules: { string } = {}
	for path, data in pairs(modules) do
		if #data.requiredBy == 0 then
			-- Exclude entry points (scripts in ServerScriptService, etc.)
			if not path:find("ServerScriptService") and not path:find("StarterPlayerScripts")
				and not path:find("StarterCharacterScripts") then
				table.insert(deadModules, path)
			end
		end
	end

	local result = {
		totalModules = 0,
		totalDependencies = 0,
		circularDependencies = circular,
		deadModules = deadModules,
		modules = {},
	}

	for path, data in pairs(modules) do
		result.totalModules += 1
		result.totalDependencies += #data.requires
		table.insert(result.modules :: any, {
			path = path,
			requiresCount = #data.requires,
			requiredByCount = #data.requiredBy,
			requires = data.requires,
			requiredBy = data.requiredBy,
		})
	end

	return true, result, nil
end
