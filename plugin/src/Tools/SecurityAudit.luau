--!strict
-- SecurityAudit: Scan place for security vulnerabilities

local TreeWalker = require(script.Parent.Parent.Utils.TreeWalker)
local HttpService = game:GetService("HttpService")

local SecurityAudit = {}

-- Stored scan results for report generation
local lastScanResults: { any }?  = nil

local function getScriptSource(script: Instance): string?
	local ok, src = pcall(function()
		return (script :: any).Source
	end)
	return if ok then src else nil
end

-- Check if a server script validates Remote arguments
local function checkRemoteValidation(serverScripts: { Instance }, remoteName: string): boolean
	for _, script in ipairs(serverScripts) do
		local source = getScriptSource(script)
		if source and source:find(remoteName) then
			-- Look for validation patterns: typeof, type, assert, if...then
			if source:find("typeof%(") or source:find("type%(") or source:find("assert%(") then
				return true
			end
			-- Check for conditional checks on the argument
			if source:find("if%s+.-then") then
				return true
			end
		end
	end
	return false
end

function SecurityAudit.scan(): (boolean, any, string?)
	local issues: { any } = {}
	local serverScripts = TreeWalker.collectScripts()

	-- 1. Check all RemoteEvents for server-side validation
	local remotes = TreeWalker.collectByClass("RemoteEvent")
	for _, remote in ipairs(remotes) do
		local hasValidation = checkRemoteValidation(serverScripts, remote.Name)
		if not hasValidation then
			table.insert(issues, {
				severity = "Critical",
				category = "Remote Validation",
				description = "RemoteEvent '" .. remote.Name .. "' has no apparent server-side input validation",
				location = remote:GetFullName(),
				suggestion = "Add typeof() checks and input validation on OnServerEvent handler",
			})
		end
	end

	-- 2. Check RemoteFunctions
	local remoteFunctions = TreeWalker.collectByClass("RemoteFunction")
	for _, rf in ipairs(remoteFunctions) do
		local hasValidation = checkRemoteValidation(serverScripts, rf.Name)
		if not hasValidation then
			table.insert(issues, {
				severity = "Critical",
				category = "Remote Validation",
				description = "RemoteFunction '" .. rf.Name .. "' has no apparent server-side input validation",
				location = rf:GetFullName(),
				suggestion = "Validate all arguments in OnServerInvoke. Consider if RemoteEvent would be safer.",
			})
		end
	end

	-- 3. Check for client-accessible sensitive ModuleScripts
	local replicatedStorage = game:GetService("ReplicatedStorage")
	TreeWalker.walkByClass(replicatedStorage, "ModuleScript", function(module)
		local source = getScriptSource(module)
		if source then
			-- Check for sensitive patterns in client-accessible modules
			local sensitivePatterns = {
				"DataStoreService", "HttpService", "TeleportService",
				"MarketplaceService:PromptPurchase", "password", "secret", "apiKey", "token",
			}
			for _, pattern in ipairs(sensitivePatterns) do
				if source:find(pattern) then
					table.insert(issues, {
						severity = "High",
						category = "Data Exposure",
						description = "Client-accessible module references '" .. pattern .. "'",
						location = module:GetFullName(),
						suggestion = "Move sensitive logic to ServerScriptService or ServerStorage",
					})
					break
				end
			end
		end
	end)

	-- 4. Check for missing rate limiting patterns
	for _, script in ipairs(serverScripts) do
		local source = getScriptSource(script)
		if source and (source:find("OnServerEvent") or source:find("OnServerInvoke")) then
			local hasRateLimit = source:find("cooldown") or source:find("rateLimit")
				or source:find("throttle") or source:find("debounce")
				or source:find("lastRequest") or source:find("lastFire")
			if not hasRateLimit then
				table.insert(issues, {
					severity = "Medium",
					category = "Rate Limiting",
					description = "Server handler has no apparent rate limiting",
					location = script:GetFullName(),
					suggestion = "Add per-player cooldown/throttle to prevent Remote spam",
				})
			end
		end
	end

	-- 5. Check for use of string-based RemoteEvent data (easily exploitable)
	for _, script in ipairs(serverScripts) do
		local source = getScriptSource(script)
		if source and source:find("loadstring") then
			table.insert(issues, {
				severity = "Critical",
				category = "Code Injection",
				description = "Script uses loadstring() which can enable code injection",
				location = script:GetFullName(),
				suggestion = "Remove loadstring() usage. Use direct function calls instead.",
			})
		end
	end

	lastScanResults = issues :: any

	local summary = {
		totalIssues = #issues,
		critical = 0,
		high = 0,
		medium = 0,
		low = 0,
		issues = issues,
	}

	for _, issue in ipairs(issues) do
		if issue.severity == "Critical" then summary.critical += 1
		elseif issue.severity == "High" then summary.high += 1
		elseif issue.severity == "Medium" then summary.medium += 1
		else summary.low += 1 end
	end

	return true, summary, nil
end

function SecurityAudit.report(): (boolean, any, string?)
	if lastScanResults == nil then
		return true, "No security scan results available. Run security_scan first.", nil
	end

	if #lastScanResults == 0 then
		return true, "=== StudioLink Security Report ===\n\nScan completed: No security issues found. Your place looks clean!", nil
	end

	local lines = { "=== StudioLink Security Report ===" }
	local grouped: { [string]: { any } } = {}

	for _, issue in ipairs(lastScanResults) do
		if not grouped[issue.severity] then
			grouped[issue.severity] = {}
		end
		table.insert(grouped[issue.severity], issue)
	end

	for _, severity in ipairs({ "Critical", "High", "Medium", "Low" }) do
		if grouped[severity] then
			table.insert(lines, "\n--- " .. severity .. " (" .. #grouped[severity] .. ") ---")
			for _, issue in ipairs(grouped[severity]) do
				table.insert(lines, "  [" .. issue.category .. "] " .. issue.description)
				table.insert(lines, "    Location: " .. issue.location)
				table.insert(lines, "    Fix: " .. issue.suggestion)
			end
		end
	end

	return true, table.concat(lines, "\n"), nil
end

return SecurityAudit
