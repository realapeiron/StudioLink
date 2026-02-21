--!strict
-- MemoryLeakScan: Detect potential memory leaks in scripts

local TreeWalker = require(script.Parent.Parent.Utils.TreeWalker)

return function(args: { [string]: any }): (boolean, any, string?)
	local issues: { any } = {}
	local scripts = TreeWalker.collectScripts()

	for _, scriptInstance in ipairs(scripts) do
		local ok, source = pcall(function() return (scriptInstance :: any).Source end)
		if not ok or not source or source == "" then continue end

		local fullName = scriptInstance:GetFullName()

		-- 1. Check for Connect without Disconnect
		local connectCount = 0
		local disconnectCount = 0
		for _ in source:gmatch(":Connect%(") do connectCount += 1 end
		for _ in source:gmatch(":Disconnect%(") do disconnectCount += 1 end
		-- Also count cleanup patterns
		for _ in source:gmatch(":Destroy%(") do disconnectCount += 1 end
		for _ in source:gmatch("%.Destroying") do disconnectCount += 1 end
		for _ in source:gmatch("Maid") do disconnectCount += 1 end
		for _ in source:gmatch("Janitor") do disconnectCount += 1 end
		for _ in source:gmatch("trove") do disconnectCount += 1 end

		if connectCount > 0 and disconnectCount == 0 then
			table.insert(issues, {
				type = "Undisconnected Connections",
				location = fullName,
				description = connectCount .. " Connect() calls with no Disconnect/Destroy/Maid/Janitor cleanup",
				severity = "High",
			})
		end

		-- 2. Check for RunService connections (high frequency = high risk)
		local runServicePatterns = {
			"Heartbeat:Connect",
			"RenderStepped:Connect",
			"Stepped:Connect",
		}
		for _, pattern in ipairs(runServicePatterns) do
			if source:find(pattern) then
				local hasCleanup = source:find(":Disconnect") or source:find(":Destroy")
					or source:find("Maid") or source:find("Janitor") or source:find("trove")
				if not hasCleanup then
					table.insert(issues, {
						type = "RunService Leak",
						location = fullName,
						description = "RunService " .. pattern .. " without cleanup — runs every frame!",
						severity = "Critical",
					})
				end
			end
		end

		-- 3. Check for Instance.new without Parent or Destroy
		local newCount = 0
		for _ in source:gmatch("Instance%.new%(") do newCount += 1 end
		local destroyCount = 0
		for _ in source:gmatch(":Destroy%(") do destroyCount += 1 end

		if newCount > 5 and destroyCount == 0 then
			table.insert(issues, {
				type = "Undestroyed Instances",
				location = fullName,
				description = newCount .. " Instance.new() calls with no :Destroy() — potential orphaned instances",
				severity = "Medium",
			})
		end

		-- 4. Check for while true do loops without task.wait
		if source:find("while%s+true%s+do") then
			if not source:find("task%.wait") and not source:find("wait%(") then
				table.insert(issues, {
					type = "Infinite Loop",
					location = fullName,
					description = "while true do loop without task.wait() — will freeze the thread",
					severity = "Critical",
				})
			end
		end

		-- 5. Check for large table accumulation patterns
		if source:find("table%.insert") and not source:find("table%.remove") and not source:find("table%.clear") then
			-- Only flag if it appears to be in a loop or connection
			if source:find(":Connect") or source:find("while") or source:find("for") then
				table.insert(issues, {
					type = "Growing Table",
					location = fullName,
					description = "table.insert in loop/connection without removal — potential unbounded growth",
					severity = "Medium",
				})
			end
		end
	end

	-- Summary
	local summary = {
		totalIssues = #issues,
		critical = 0,
		high = 0,
		medium = 0,
		issues = issues,
	}

	for _, issue in ipairs(issues) do
		if issue.severity == "Critical" then summary.critical += 1
		elseif issue.severity == "High" then summary.high += 1
		else summary.medium += 1 end
	end

	return true, summary, nil
end
