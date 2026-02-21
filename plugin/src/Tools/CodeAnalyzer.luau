--!strict
-- CodeAnalyzer: Luau code quality linter

local TreeWalker = require(script.Parent.Parent.Utils.TreeWalker)

return function(args: { [string]: any }): (boolean, any, string?)
	local targetPath = args.path or ""
	local issues: { any } = {}
	local scripts = TreeWalker.collectScripts()

	for _, scriptInstance in ipairs(scripts) do
		if targetPath ~= "" and not scriptInstance:GetFullName():find(targetPath) then
			continue
		end

		local ok, source = pcall(function() return (scriptInstance :: any).Source end)
		if not ok or not source or source == "" then continue end

		local fullName = scriptInstance:GetFullName()
		local lineNum = 0

		for line in source:gmatch("[^\r\n]+") do
			lineNum += 1

			-- 1. Deprecated API usage
			if line:find("^%s*wait%(") or line:find("[^%w_.]wait%(") then
				if not line:find("task%.wait") then
					table.insert(issues, {
						rule = "deprecated-wait",
						severity = "Warning",
						line = lineNum,
						message = "Use task.wait() instead of wait()",
						scriptPath = fullName,
					})
				end
			end

			if line:find("^%s*spawn%(") or line:find("[^%w_.]spawn%(") then
				if not line:find("task%.spawn") then
					table.insert(issues, {
						rule = "deprecated-spawn",
						severity = "Warning",
						line = lineNum,
						message = "Use task.spawn() instead of spawn()",
						scriptPath = fullName,
					})
				end
			end

			if line:find("^%s*delay%(") or line:find("[^%w_.]delay%(") then
				if not line:find("task%.delay") then
					table.insert(issues, {
						rule = "deprecated-delay",
						severity = "Warning",
						line = lineNum,
						message = "Use task.delay() instead of delay()",
						scriptPath = fullName,
					})
				end
			end

			-- 2. Global variable detection (assignments without local)
			if line:find("^%s*%a[%w_]*%s*=") and not line:find("^%s*local")
				and not line:find("^%s*%-%-") and not line:find("%.")
				and not line:find(":") and not line:find("%[") then
				-- Exclude common patterns
				local varName = line:match("^%s*(%a[%w_]*)%s*=")
				if varName and varName ~= "self" and varName ~= "module" then
					table.insert(issues, {
						rule = "global-variable",
						severity = "Warning",
						line = lineNum,
						message = "Possible global variable '" .. varName .. "' — use 'local'",
						scriptPath = fullName,
					})
				end
			end

			-- 3. String concatenation in loops (performance)
			if (line:find("%.%.") and (line:find("for") or line:find("while"))) then
				table.insert(issues, {
					rule = "string-concat-loop",
					severity = "Info",
					line = lineNum,
					message = "String concatenation in loop — consider table.concat()",
					scriptPath = fullName,
				})
			end

			-- 4. pairs() vs ipairs() for arrays
			if line:find("pairs%(") and not line:find("ipairs%(") then
				-- This is just informational
			end

			-- 5. Deprecated services/methods
			if line:find("game%.Lighting") and not line:find("GetService") then
				table.insert(issues, {
					rule = "direct-service-access",
					severity = "Info",
					line = lineNum,
					message = "Use game:GetService('Lighting') instead of game.Lighting",
					scriptPath = fullName,
				})
			end
		end

		-- Script-level checks

		-- 6. Missing --!strict
		if not source:find("^%-%-!strict") then
			table.insert(issues, {
				rule = "missing-strict",
				severity = "Info",
				line = 1,
				message = "Consider adding --!strict for type checking",
				scriptPath = fullName,
			})
		end

		-- 7. Very long scripts
		local lineCount = select(2, source:gsub("\n", "")) + 1
		if lineCount > 500 then
			table.insert(issues, {
				rule = "long-script",
				severity = "Info",
				line = 1,
				message = "Script has " .. lineCount .. " lines — consider splitting into modules",
				scriptPath = fullName,
			})
		end
	end

	-- Group by severity
	local summary = {
		totalIssues = #issues,
		warnings = 0,
		info = 0,
		errors = 0,
		issues = issues,
	}

	for _, issue in ipairs(issues) do
		if issue.severity == "Warning" then summary.warnings += 1
		elseif issue.severity == "Info" then summary.info += 1
		else summary.errors += 1 end
	end

	return true, summary, nil
end
