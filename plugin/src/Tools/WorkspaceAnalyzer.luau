--!strict
-- WorkspaceAnalyzer: Comprehensive workspace analysis tool
-- Analyzes coding style, architecture, statistics, issues, dependencies, and patterns

local TreeWalker = require(script.Parent.Parent.Utils.TreeWalker)

-- ═══════════════════════════════════════════
-- STYLE ANALYSIS
-- ═══════════════════════════════════════════

local function analyzeStyle(scripts: { Instance }): { [string]: any }
	local camelCase = 0
	local pascalCase = 0
	local snakeCase = 0
	local tabIndent = 0
	local spaceIndent = 0
	local strictCount = 0
	local typeAnnotations = 0
	local totalFunctions = 0
	local totalFunctionLines = 0
	local totalCommentLines = 0
	local totalLines = 0
	local scriptsWithTypes = 0

	for _, scriptInstance in ipairs(scripts) do
		local ok, source = pcall(function() return (scriptInstance :: any).Source end)
		if not ok or not source or source == "" then continue end

		local lineCount = select(2, source:gsub("\n", "")) + 1
		totalLines += lineCount

		-- Strict mode
		if source:find("^%-%-!strict") then
			strictCount += 1
		end

		-- Type annotations (: type after parameters)
		local typeCount = 0
		for _ in source:gmatch(":%s*[A-Z][%w_%.]*[%?]?") do
			typeCount += 1
		end
		if typeCount > 0 then
			scriptsWithTypes += 1
		end
		typeAnnotations += typeCount

		local inFunction = false
		local funcStartLine = 0

		for line in source:gmatch("[^\r\n]+") do
			-- Indent style
			if line:find("^\t") then
				tabIndent += 1
			elseif line:find("^  ") then
				spaceIndent += 1
			end

			-- Comments
			if line:find("^%s*%-%-") then
				totalCommentLines += 1
			end

			-- Function tracking
			if line:find("function%s+[%w_%.:]") or line:find("function%(") then
				totalFunctions += 1
				if not inFunction then
					inFunction = true
					funcStartLine = 0
				end
			end
			if inFunction then
				funcStartLine += 1
				if line:find("^%s*end") then
					totalFunctionLines += funcStartLine
					inFunction = false
				end
			end

			-- Naming conventions (from local declarations)
			local varName = line:match("^%s*local%s+function%s+(%a[%w_]*)")
				or line:match("^%s*local%s+(%a[%w_]*)%s*=")
			if varName and #varName > 2 then
				if varName:find("^%l") and varName:find("%u") and not varName:find("_") then
					camelCase += 1
				elseif varName:find("^%u") and not varName:find("_") then
					pascalCase += 1
				elseif varName:find("_") and varName == varName:lower() then
					snakeCase += 1
				end
			end
		end
	end

	local totalNamed = camelCase + pascalCase + snakeCase
	local dominantNaming = "mixed"
	if totalNamed > 0 then
		if camelCase / totalNamed > 0.6 then dominantNaming = "camelCase"
		elseif pascalCase / totalNamed > 0.6 then dominantNaming = "PascalCase"
		elseif snakeCase / totalNamed > 0.6 then dominantNaming = "snake_case"
		end
	end

	return {
		namingConvention = dominantNaming,
		namingBreakdown = {
			camelCase = camelCase,
			PascalCase = pascalCase,
			snake_case = snakeCase,
		},
		indentStyle = if tabIndent > spaceIndent then "tabs" else "spaces",
		indentBreakdown = { tabs = tabIndent, spaces = spaceIndent },
		strictModeUsage = strictCount .. "/" .. #scripts,
		strictModePercent = if #scripts > 0 then math.floor(strictCount / #scripts * 100) else 0,
		typeAnnotationCount = typeAnnotations,
		scriptsWithTypes = scriptsWithTypes .. "/" .. #scripts,
		avgFunctionLength = if totalFunctions > 0 then math.floor(totalFunctionLines / totalFunctions) else 0,
		commentRatio = if totalLines > 0 then math.floor(totalCommentLines / totalLines * 100) .. "%" else "0%",
		totalCommentLines = totalCommentLines,
	}
end

-- ═══════════════════════════════════════════
-- ARCHITECTURE ANALYSIS
-- ═══════════════════════════════════════════

local function analyzeArchitecture(scripts: { Instance }): { [string]: any }
	local framework = "None"
	local frameworkDetails: { string } = {}
	local serviceCount = 0
	local controllerCount = 0
	local moduleCount = 0
	local folderStructure: { [string]: number } = {}
	local entryPoints: { string } = {}

	for _, scriptInstance in ipairs(scripts) do
		local ok, source = pcall(function() return (scriptInstance :: any).Source end)
		if not ok or not source or source == "" then continue end

		local fullName = scriptInstance:GetFullName()

		-- Framework detection
		if source:find("Knit%.CreateService") or source:find("Knit%.CreateController") then
			framework = "Knit"
			if source:find("Knit%.CreateService") then
				serviceCount += 1
			end
			if source:find("Knit%.CreateController") then
				controllerCount += 1
			end
		end
		if source:find("ProfileService") or source:find("ProfileStore") then
			table.insert(frameworkDetails, "ProfileService")
		end

		-- Folder structure stats
		local topLevel = fullName:match("^game%.(%w+)") or "Other"
		folderStructure[topLevel] = (folderStructure[topLevel] or 0) + 1

		-- Entry points (non-module scripts in SSS or StarterPlayer)
		if scriptInstance:IsA("Script") and not scriptInstance:IsA("ModuleScript") then
			if fullName:find("ServerScriptService") or fullName:find("StarterPlayer") or fullName:find("StarterGui") then
				table.insert(entryPoints, fullName)
			end
		end

		-- Module counting
		if scriptInstance:IsA("ModuleScript") then
			moduleCount += 1
		end
	end

	-- Deduplicate framework details
	local seen: { [string]: boolean } = {}
	local uniqueFrameworkDetails: { string } = {}
	for _, detail in ipairs(frameworkDetails) do
		if not seen[detail] then
			seen[detail] = true
			table.insert(uniqueFrameworkDetails, detail)
		end
	end

	return {
		framework = framework,
		frameworkLibraries = uniqueFrameworkDetails,
		knitServices = serviceCount,
		knitControllers = controllerCount,
		moduleCount = moduleCount,
		entryPoints = entryPoints,
		folderDistribution = folderStructure,
	}
end

-- ═══════════════════════════════════════════
-- STATISTICS
-- ═══════════════════════════════════════════

local function analyzeStatistics(scripts: { Instance }): { [string]: any }
	local serverScripts = 0
	local localScripts = 0
	local moduleScripts = 0
	local totalLines = 0
	local longestScript = ""
	local longestScriptLines = 0
	local emptyScripts: { string } = {}

	for _, scriptInstance in ipairs(scripts) do
		-- Count by type
		if scriptInstance:IsA("ModuleScript") then
			moduleScripts += 1
		elseif scriptInstance:IsA("LocalScript") then
			localScripts += 1
		elseif scriptInstance:IsA("Script") then
			serverScripts += 1
		end

		local ok, source = pcall(function() return (scriptInstance :: any).Source end)
		if not ok or not source then continue end

		if source == "" then
			table.insert(emptyScripts, scriptInstance:GetFullName())
			continue
		end

		local lineCount = select(2, source:gsub("\n", "")) + 1
		totalLines += lineCount

		if lineCount > longestScriptLines then
			longestScriptLines = lineCount
			longestScript = scriptInstance:GetFullName()
		end
	end

	local totalScripts = serverScripts + localScripts + moduleScripts

	return {
		totalScripts = totalScripts,
		serverScripts = serverScripts,
		localScripts = localScripts,
		moduleScripts = moduleScripts,
		totalLines = totalLines,
		avgLinesPerScript = if totalScripts > 0 then math.floor(totalLines / totalScripts) else 0,
		longestScript = longestScript,
		longestScriptLines = longestScriptLines,
		emptyScripts = emptyScripts,
		emptyScriptCount = #emptyScripts,
	}
end

-- ═══════════════════════════════════════════
-- ISSUES ANALYSIS (lint + security + memory + optimization)
-- ═══════════════════════════════════════════

local function analyzeIssues(scripts: { Instance }): { [string]: any }
	local issues: { any } = {}
	local categories = { deprecated = 0, security = 0, memory = 0, optimization = 0, style = 0 }

	for _, scriptInstance in ipairs(scripts) do
		local ok, source = pcall(function() return (scriptInstance :: any).Source end)
		if not ok or not source or source == "" then continue end

		local fullName = scriptInstance:GetFullName()
		local lineNum = 0

		for line in source:gmatch("[^\r\n]+") do
			lineNum += 1

			-- DEPRECATED APIs
			if (line:find("^%s*wait%(") or line:find("[^%w_.]wait%(")) and not line:find("task%.wait") then
				table.insert(issues, { category = "deprecated", severity = "Warning", message = "wait() → task.wait()", scriptPath = fullName, line = lineNum })
				categories.deprecated += 1
			end
			if (line:find("^%s*spawn%(") or line:find("[^%w_.]spawn%(")) and not line:find("task%.spawn") then
				table.insert(issues, { category = "deprecated", severity = "Warning", message = "spawn() → task.spawn()", scriptPath = fullName, line = lineNum })
				categories.deprecated += 1
			end
			if (line:find("^%s*delay%(") or line:find("[^%w_.]delay%(")) and not line:find("task%.delay") then
				table.insert(issues, { category = "deprecated", severity = "Warning", message = "delay() → task.delay()", scriptPath = fullName, line = lineNum })
				categories.deprecated += 1
			end

			-- SECURITY
			if line:find("loadstring") then
				table.insert(issues, { category = "security", severity = "Critical", message = "loadstring() usage detected — code injection risk", scriptPath = fullName, line = lineNum })
				categories.security += 1
			end

			-- RemoteEvent/Function without validation (server-side)
			if line:find("%.OnServerEvent:Connect") or line:find("%.OnServerInvoke%s*=") then
				-- Check if there's any validation nearby (simple heuristic)
				local nearbyLines = source:sub(math.max(1, source:find(line, 1, true) or 1), math.min(#source, (source:find(line, 1, true) or 1) + 500))
				if not nearbyLines:find("typeof") and not nearbyLines:find("type%(") and not nearbyLines:find("assert") then
					table.insert(issues, { category = "security", severity = "High", message = "Remote handler without input validation", scriptPath = fullName, line = lineNum })
					categories.security += 1
				end
			end

			-- MEMORY
			if line:find(":Connect%(") then
				local nearbySource = source
				if not nearbySource:find(":Disconnect") and not nearbySource:find(":Destroy")
					and not nearbySource:find("Maid") and not nearbySource:find("Janitor")
					and not nearbySource:find("Trove") and not nearbySource:find("trove") then
					-- Only flag if it's a RunService connection (high risk)
					if line:find("Heartbeat") or line:find("RenderStepped") or line:find("Stepped") then
						table.insert(issues, { category = "memory", severity = "High", message = "RunService connection without cleanup mechanism", scriptPath = fullName, line = lineNum })
						categories.memory += 1
					end
				end
			end

			-- OPTIMIZATION
			if line:find("%.%.") and (source:sub(1, source:find(line, 1, true) or 1):find("for") or source:sub(1, source:find(line, 1, true) or 1):find("while")) then
				-- Only flag if it looks like it's inside a loop
				if line:find("for%s") or line:find("while%s") then
					table.insert(issues, { category = "optimization", severity = "Info", message = "String concatenation in loop — use table.concat()", scriptPath = fullName, line = lineNum })
					categories.optimization += 1
				end
			end
		end

		-- Script-level style checks
		if not source:find("^%-%-!strict") then
			categories.style += 1
		end
	end

	-- Cap issues to 50 for readability
	local cappedIssues = issues
	local totalIssues = #issues
	if #issues > 50 then
		cappedIssues = {}
		for i = 1, 50 do
			table.insert(cappedIssues, issues[i])
		end
	end

	return {
		totalIssues = totalIssues,
		categories = categories,
		issues = cappedIssues,
		truncated = totalIssues > 50,
	}
end

-- ═══════════════════════════════════════════
-- DEPENDENCY ANALYSIS
-- ═══════════════════════════════════════════

local function analyzeDependencies(scripts: { Instance }): { [string]: any }
	local modules: { [string]: { requires: { string }, requiredBy: { string } } } = {}
	local moduleIndex: { [string]: string } = {} -- path -> fullName

	-- Build module index
	for _, scriptInstance in ipairs(scripts) do
		if scriptInstance:IsA("ModuleScript") then
			local fullName = scriptInstance:GetFullName()
			modules[fullName] = { requires = {}, requiredBy = {} }
			moduleIndex[scriptInstance.Name] = fullName
		end
	end

	-- Find require() calls
	for _, scriptInstance in ipairs(scripts) do
		local ok, source = pcall(function() return (scriptInstance :: any).Source end)
		if not ok or not source or source == "" then continue end

		local callerName = scriptInstance:GetFullName()

		for reqTarget in source:gmatch("require%([^)]*%.([%w_]+)%)") do
			local targetFullName = moduleIndex[reqTarget]
			if targetFullName and modules[targetFullName] then
				table.insert(modules[targetFullName].requiredBy, callerName)
				if modules[callerName] then
					table.insert(modules[callerName].requires, targetFullName)
				end
			end
		end
	end

	-- Find dead modules (never required, excluding entry points)
	local deadModules: { string } = {}
	for fullName, data in pairs(modules) do
		if #data.requiredBy == 0 then
			-- Exclude common entry-point-like modules
			if not fullName:find("init") and not fullName:find("Init") then
				table.insert(deadModules, fullName)
			end
		end
	end

	-- Simple circular dependency check (depth 1)
	local circularDeps: { { string } } = {}
	for nameA, dataA in pairs(modules) do
		for _, reqB in ipairs(dataA.requires) do
			if modules[reqB] then
				for _, reqBack in ipairs(modules[reqB].requires) do
					if reqBack == nameA then
						table.insert(circularDeps, { nameA, reqB })
					end
				end
			end
		end
	end

	-- Top required modules
	local topModules: { { name: string, count: number } } = {}
	for fullName, data in pairs(modules) do
		if #data.requiredBy > 1 then
			table.insert(topModules, { name = fullName, count = #data.requiredBy })
		end
	end
	table.sort(topModules, function(a, b) return a.count > b.count end)
	-- Cap to top 10
	if #topModules > 10 then
		local capped = {}
		for i = 1, 10 do table.insert(capped, topModules[i]) end
		topModules = capped
	end

	return {
		totalModules = 0, -- will be counted below
		deadModules = deadModules,
		deadModuleCount = #deadModules,
		circularDependencies = circularDeps,
		circularCount = #circularDeps,
		mostRequired = topModules,
	}
end

-- ═══════════════════════════════════════════
-- PATTERN DETECTION
-- ═══════════════════════════════════════════

local function analyzePatterns(scripts: { Instance }): { [string]: any }
	local libraries: { [string]: boolean } = {}
	local remoteNames: { string } = {}
	local remoteNamingStyle = "unknown"
	local usesPromises = false
	local usesSignal = false

	for _, scriptInstance in ipairs(scripts) do
		local ok, source = pcall(function() return (scriptInstance :: any).Source end)
		if not ok or not source or source == "" then continue end

		-- Library detection
		if source:find("Knit") then libraries["Knit"] = true end
		if source:find("ProfileService") or source:find("ProfileStore") then libraries["ProfileService"] = true end
		if source:find("Maid") then libraries["Maid"] = true end
		if source:find("Janitor") then libraries["Janitor"] = true end
		if source:find("Trove") or source:find("trove") then libraries["Trove"] = true end
		if source:find("Promise") then libraries["Promise"] = true; usesPromises = true end
		if source:find("Signal") or source:find("GoodSignal") then libraries["Signal"] = true; usesSignal = true end
		if source:find("Comm") and source:find("Knit") then libraries["Knit.Comm"] = true end
		if source:find("ReplicaService") or source:find("ReplicaController") then libraries["ReplicaService"] = true end
		if source:find("Rodux") then libraries["Rodux"] = true end
		if source:find("Roact") then libraries["Roact"] = true end
		if source:find("React") and source:find("createElement") then libraries["React"] = true end
		if source:find("Fusion") then libraries["Fusion"] = true end
		if source:find("Cmdr") then libraries["Cmdr"] = true end
		if source:find("Wally") or source:find("wally%.toml") then libraries["Wally"] = true end
		if source:find("TestEZ") or source:find("describe%(") then libraries["TestEZ"] = true end
		if source:find("DataStore2") then libraries["DataStore2"] = true end
		if source:find("Zone") and source:find("ZonePlus") then libraries["ZonePlus"] = true end
		if source:find("BridgeNet") then libraries["BridgeNet"] = true end
		if source:find("Red") and source:find("Red%.Call") then libraries["Red"] = true end

		-- Remote naming patterns
		for remoteName in source:gmatch("RemoteEvent%(%s*\"([^\"]+)\"") do
			table.insert(remoteNames, remoteName)
		end
		for remoteName in source:gmatch("RemoteFunction%(%s*\"([^\"]+)\"") do
			table.insert(remoteNames, remoteName)
		end
	end

	-- Determine remote naming style
	if #remoteNames > 0 then
		local camel, pascal, snake = 0, 0, 0
		for _, name in ipairs(remoteNames) do
			if name:find("^%l") and name:find("%u") then camel += 1
			elseif name:find("^%u") then pascal += 1
			elseif name:find("_") then snake += 1
			end
		end
		local total = camel + pascal + snake
		if total > 0 then
			if camel / total > 0.5 then remoteNamingStyle = "camelCase"
			elseif pascal / total > 0.5 then remoteNamingStyle = "PascalCase"
			elseif snake / total > 0.5 then remoteNamingStyle = "snake_case"
			end
		end
	end

	-- Convert libraries set to array
	local libraryList: { string } = {}
	for lib in pairs(libraries) do
		table.insert(libraryList, lib)
	end
	table.sort(libraryList)

	return {
		libraries = libraryList,
		libraryCount = #libraryList,
		remoteNamingConvention = remoteNamingStyle,
		remoteCount = #remoteNames,
		usesPromises = usesPromises,
		usesSignalPattern = usesSignal,
	}
end

-- ═══════════════════════════════════════════
-- MAIN ENTRY POINT
-- ═══════════════════════════════════════════

return function(args: { [string]: any }): (boolean, any, string?)
	local targetPath = args.path or ""
	local scripts = TreeWalker.collectScripts()

	-- Filter by path if specified
	if targetPath ~= "" then
		local filtered: { Instance } = {}
		for _, s in ipairs(scripts) do
			if s:GetFullName():find(targetPath) then
				table.insert(filtered, s)
			end
		end
		scripts = filtered
	end

	if #scripts == 0 then
		return true, { error = "No scripts found" .. (if targetPath ~= "" then " matching path: " .. targetPath else "") }, nil
	end

	local report = {
		style = analyzeStyle(scripts),
		architecture = analyzeArchitecture(scripts),
		statistics = analyzeStatistics(scripts),
		issues = analyzeIssues(scripts),
		dependencies = analyzeDependencies(scripts),
		patterns = analyzePatterns(scripts),
		analyzedScripts = #scripts,
		path = if targetPath ~= "" then targetPath else "entire workspace",
	}

	return true, report, nil
end
