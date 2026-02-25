--!strict
-- ScriptTools: Script source management and search utilities for StudioLink

local Serializer = require(script.Parent.Parent.Utils.Serializer)
local TreeWalker = require(script.Parent.Parent.Utils.TreeWalker)

local ScriptTools = {}

-- ═══════════════════════════════════════════
-- TOOL IMPLEMENTATIONS
-- ═══════════════════════════════════════════

-- Resolve a dot-separated path to an Instance
local function resolvePath(path: string): Instance?
	local parts = path:split(".")
	local current: Instance = game

	local startIndex = 1
	if parts[1] == "game" then
		startIndex = 2
	end

	for i = startIndex, #parts do
		local child = current:FindFirstChild(parts[i])
		if not child then
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

-- Get script source with line numbers
function ScriptTools.getScriptSource(args: { [string]: any }): (boolean, any, string?)
	local path = args.path
	if not path or path == "" then
		return false, nil, "Missing required parameter: path"
	end

	local instance = resolvePath(path)
	if not instance then
		return false, nil, "Instance not found: " .. path
	end

	if not instance:IsA("LuaSourceContainer") then
		return false, nil, "Instance is not a script: " .. instance.ClassName
	end

	local ok, source = pcall(function()
		return (instance :: any).Source
	end)

	if not ok then
		return false, nil, "Failed to read script source: " .. tostring(source)
	end

	-- Add line numbers
	local lines = source:split("\n")
	local numbered = {}
	for i, line in ipairs(lines) do
		table.insert(numbered, string.format("%4d | %s", i, line))
	end

	return true, {
		path = instance:GetFullName(),
		className = instance.ClassName,
		lineCount = #lines,
		source = table.concat(numbered, "\n"),
	}, nil
end

-- Set script source
function ScriptTools.setScriptSource(args: { [string]: any }): (boolean, any, string?)
	local path = args.path
	local source = args.source

	if not path or not source then
		return false, nil, "Missing required parameters: path, source"
	end

	local instance = resolvePath(path)
	if not instance then
		return false, nil, "Instance not found: " .. path
	end

	if not instance:IsA("LuaSourceContainer") then
		return false, nil, "Instance is not a script: " .. instance.ClassName
	end

	-- Record old line count for comparison
	local oldSource = ""
	pcall(function() oldSource = (instance :: any).Source end)
	local oldLineCount = #oldSource:split("\n")

	local ok, err = pcall(function()
		(instance :: any).Source = source
	end)

	if not ok then
		return false, nil, "Failed to set script source: " .. tostring(err)
	end

	local newLineCount = #source:split("\n")

	-- Record waypoint for undo
	pcall(function()
		local CHS = game:GetService("ChangeHistoryService")
		;(CHS :: any):SetWaypoint("StudioLink: Edit " .. instance.Name)
	end)

	return true, {
		path = instance:GetFullName(),
		oldLineCount = oldLineCount,
		newLineCount = newLineCount,
		updated = true,
	}, nil
end

-- Grep scripts: search all scripts for a pattern
function ScriptTools.grepScripts(args: { [string]: any }): (boolean, any, string?)
	local pattern = args.pattern
	if not pattern or pattern == "" then
		return false, nil, "Missing required parameter: pattern"
	end

	local caseSensitive = args.caseSensitive
	if caseSensitive == nil then caseSensitive = true end

	local searchPattern = pattern
	if not caseSensitive then
		searchPattern = pattern:lower()
	end

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

	local results = {}
	local scriptsSearched = 0
	local maxResults = 100

	for _, service in ipairs(services) do
		if #results >= maxResults then break end

		for _, desc in ipairs(service:GetDescendants()) do
			if #results >= maxResults then break end

			if desc:IsA("LuaSourceContainer") then
				scriptsSearched += 1
				local ok, source = pcall(function()
					return (desc :: any).Source
				end)

				if ok and source and source ~= "" then
					local searchSource = if caseSensitive then source else source:lower()
					local lines = source:split("\n")
					local searchLines = searchSource:split("\n")
					local matchingLines = {}

					for i, searchLine in ipairs(searchLines) do
						if searchLine:find(searchPattern, 1, true) then
							table.insert(matchingLines, {
								line = i,
								content = lines[i]:sub(1, 200), -- trim long lines
							})
						end
					end

					if #matchingLines > 0 then
						table.insert(results, {
							path = desc:GetFullName(),
							className = desc.ClassName,
							matches = matchingLines,
							matchCount = #matchingLines,
						})
					end
				end
			end
		end
	end

	return true, {
		pattern = pattern,
		caseSensitive = caseSensitive,
		scriptsSearched = scriptsSearched,
		filesMatched = #results,
		results = results,
		truncated = #results >= maxResults,
	}, nil
end

-- Search objects: find instances by name or class
function ScriptTools.searchObjects(args: { [string]: any }): (boolean, any, string?)
	local query = args.query
	if not query or query == "" then
		return false, nil, "Missing required parameter: query"
	end

	local searchBy = args.searchBy or "name" -- "name", "class", or "both"

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

	local results = {}
	local maxResults = 100
	local queryLower = query:lower()

	for _, service in ipairs(services) do
		if #results >= maxResults then break end

		for _, desc in ipairs(service:GetDescendants()) do
			if #results >= maxResults then break end

			local match = false

			if searchBy == "name" or searchBy == "both" then
				if desc.Name:lower():find(queryLower, 1, true) then
					match = true
				end
			end

			if searchBy == "class" or searchBy == "both" then
				if desc.ClassName:lower():find(queryLower, 1, true) then
					match = true
				end
			end

			if match then
				table.insert(results, {
					path = desc:GetFullName(),
					name = desc.Name,
					className = desc.ClassName,
					childCount = #desc:GetChildren(),
				})
			end
		end
	end

	return true, {
		query = query,
		searchBy = searchBy,
		resultCount = #results,
		results = results,
		truncated = #results >= maxResults,
	}, nil
end

return ScriptTools
