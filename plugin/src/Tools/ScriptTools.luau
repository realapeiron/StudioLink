--!strict
-- ScriptTools: Script source management and search utilities for StudioLink

local Serializer = require(script.Parent.Parent.Utils.Serializer)
local TreeWalker = require(script.Parent.Parent.Utils.TreeWalker)
local PathResolver = require(script.Parent.Parent.Utils.PathResolver)
local LineEdit = require(script.Parent.Parent.Utils.LineEdit)
local FindReplace = require(script.Parent.Parent.Utils.FindReplace)

local ScriptTools = {}

-- ═══════════════════════════════════════════
-- TOOL IMPLEMENTATIONS
-- ═══════════════════════════════════════════

local function resolvePath(path: string): Instance?
	return PathResolver.resolve(path)
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

	-- Record waypoint BEFORE change (so undo reverts to pre-change state)
	pcall(function()
		local CHS = game:GetService("ChangeHistoryService")
		;(CHS :: any):SetWaypoint("StudioLink: Edit " .. instance.Name)
	end)

	local ok, err = pcall(function()
		(instance :: any).Source = source
	end)

	if not ok then
		return false, nil, "Failed to set script source: " .. tostring(err)
	end

	local newLineCount = #source:split("\n")

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

-- ───────────────────────── Surgical script editing (v0.8.0) ─────────────────────────

local function readSource(path: any): (Instance?, string?, string?)
	if type(path) ~= "string" or path == "" then
		return nil, nil, "path is required"
	end
	local instance = resolvePath(path)
	if not instance then
		return nil, nil, "Script not found: " .. path
	end
	local source: string? = nil
	local ok = pcall(function()
		source = (instance :: any).Source
	end)
	if not ok or source == nil then
		return nil, nil, "could not read .Source (is '" .. path .. "' a Script/LocalScript/ModuleScript?)"
	end
	return instance, source, nil
end

local function writeSource(instance: Instance, newSource: string): (boolean, string?)
	local ok, err = pcall(function()
		(instance :: any).Source = newSource
	end)
	if not ok then
		return false, "could not write .Source: " .. tostring(err)
	end
	return true, nil
end

function ScriptTools.editScriptLines(args: { [string]: any }): (boolean, any, string?)
	if type(args.old_string) ~= "string" or args.old_string == "" then
		return false, nil, "old_string is required"
	end
	if type(args.new_string) ~= "string" then
		return false, nil, "new_string is required (string)"
	end
	local instance, source, err = readSource(args.path)
	if not instance or source == nil then
		return false, nil, err
	end
	local startLine = if type(args.start_line) == "number" then args.start_line else nil
	local newSource, replaceErr = LineEdit.replaceText(source, args.old_string, args.new_string, startLine)
	if newSource == nil then
		return false, nil, replaceErr
	end
	local wOk, wErr = writeSource(instance, newSource)
	if not wOk then
		return false, nil, wErr
	end
	return true, { path = args.path, edited = true }, nil
end

function ScriptTools.insertScriptLines(args: { [string]: any }): (boolean, any, string?)
	if type(args.after_line) ~= "number" then
		return false, nil, "after_line is required (number; 0 = before the first line)"
	end
	if type(args.content) ~= "string" then
		return false, nil, "content is required (string)"
	end
	local instance, source, err = readSource(args.path)
	if not instance or source == nil then
		return false, nil, err
	end
	local wOk, wErr = writeSource(instance, LineEdit.insertAfter(source, args.after_line, args.content))
	if not wOk then
		return false, nil, wErr
	end
	return true, { path = args.path, inserted = true }, nil
end

function ScriptTools.deleteScriptLines(args: { [string]: any }): (boolean, any, string?)
	if type(args.start_line) ~= "number" or type(args.end_line) ~= "number" then
		return false, nil, "start_line and end_line are required (numbers, 1-indexed inclusive)"
	end
	local instance, source, err = readSource(args.path)
	if not instance or source == nil then
		return false, nil, err
	end
	local wOk, wErr = writeSource(instance, LineEdit.deleteRange(source, args.start_line, args.end_line))
	if not wOk then
		return false, nil, wErr
	end
	return true, { path = args.path, deleted = true }, nil
end

function ScriptTools.findAndReplaceInScripts(args: { [string]: any }): (boolean, any, string?)
	local pattern = args.pattern
	if type(pattern) ~= "string" or pattern == "" then
		return false, nil, "pattern is required"
	end
	if type(args.replacement) ~= "string" then
		return false, nil, "replacement is required (string)"
	end
	local usePattern = args.usePattern == true
	local dryRun = args.dryRun == true
	local maxReplacements = if type(args.maxReplacements) == "number" then args.maxReplacements else 1000
	local pathFilter = if type(args.path) == "string" and args.path ~= "" then args.path else nil

	local changes: { any } = {}
	local total = 0
	for _, scriptInstance in ipairs(TreeWalker.collectScripts()) do
		if pathFilter and not scriptInstance:GetFullName():find(pathFilter, 1, true) then
			continue
		end
		local ok, source = pcall(function()
			return (scriptInstance :: any).Source
		end)
		if not ok or type(source) ~= "string" then
			continue
		end
		local newSource, count = FindReplace.replaceInSource(source, pattern, args.replacement, usePattern)
		if count > 0 then
			total += count
			table.insert(changes, { path = scriptInstance:GetFullName(), replacements = count })
			if not dryRun then
				pcall(function()
					(scriptInstance :: any).Source = newSource
				end)
			end
			if total >= maxReplacements then
				break
			end
		end
	end
	return true,
		{ dryRun = dryRun, totalReplacements = total, scriptsAffected = #changes, changes = changes },
		nil
end

return ScriptTools
