--!strict
-- script_patch — wrap set_script_source with optional syntax check (when
-- loadstring is available), diff stats, and ChangeHistoryService waypoints.
-- This is NOT live hot-reload: existing required ModuleScripts in running
-- scripts continue using the old version. Source applies on the next
-- require() / play start.

local PathResolver = require(script.Parent.Parent.Utils.PathResolver)

local function countLines(s: string): number
	if s == "" then
		return 0
	end
	local count = 1
	for _ in s:gmatch("\n") do
		count = count + 1
	end
	return count
end

return function(args: { [string]: any }): (boolean, any, string?)
	local path = args.module_path or args.path
	local newSource = args.new_source or args.source

	if type(path) ~= "string" or path == "" then
		return false, nil, "module_path is required (string)"
	end
	if type(newSource) ~= "string" then
		return false, nil, "new_source is required (string)"
	end

	local instance = PathResolver.resolve(path)
	if not instance then
		return false, nil, "instance not found: " .. path
	end

	if
		not (
			instance:IsA("ModuleScript")
			or instance:IsA("Script")
			or instance:IsA("LocalScript")
		)
	then
		return false,
			nil,
			"instance is " .. instance.ClassName .. ", not a Script / LocalScript / ModuleScript"
	end

	-- Optional syntax check: loadstring is usually disabled in plugin context,
	-- but we try in case the user has it on. Atomic abort on failure.
	local syntaxChecked = false
	local loadstringFn: any = (rawget(getfenv(0), "loadstring") :: any) or nil
	if type(loadstringFn) == "function" then
		local fn, err = loadstringFn(newSource)
		if not fn then
			return false, nil, "syntax error in new_source: " .. tostring(err)
		end
		syntaxChecked = true
	end

	local oldSource: string = (instance :: any).Source
	local oldLines = countLines(oldSource)
	local newLines = countLines(newSource)

	local CHS = game:GetService("ChangeHistoryService") :: any
	pcall(function()
		CHS:SetWaypoint("script_patch: before")
	end)
	;(instance :: any).Source = newSource
	pcall(function()
		CHS:SetWaypoint("script_patch: after")
	end)

	local warnings: { string } = {}
	if not syntaxChecked then
		table.insert(
			warnings,
			"syntax check skipped (loadstring unavailable in plugin context); source applied unchecked"
		)
	end
	if instance:IsA("ModuleScript") then
		table.insert(
			warnings,
			"ModuleScript: existing required references continue using the old version. Source takes effect on next require() / play restart."
		)
	end

	return true,
		{
			path = instance:GetFullName(),
			class_name = instance.ClassName,
			old_line_count = oldLines,
			new_line_count = newLines,
			line_delta = newLines - oldLines,
			syntax_checked = syntaxChecked,
			warnings = warnings,
		},
		nil
end
