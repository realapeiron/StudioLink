--!strict
-- NetworkMonitor: Monitor RemoteEvent/Function traffic

local TreeWalker = require(script.Parent.Parent.Utils.TreeWalker)
local HttpService = game:GetService("HttpService")

local NetworkMonitor = {}

local monitoring = false
local connections: { RBXScriptConnection } = {}
local trafficLog: { [string]: { count: number, totalSize: number, timestamps: { number } } } = {}
local startTime = 0

function NetworkMonitor.start(args: { [string]: any }): (boolean, any, string?)
	if monitoring then
		return false, nil, "Network monitor is already running. Stop it first."
	end

	monitoring = true
	trafficLog = {}
	startTime = os.clock()

	-- Monitor all RemoteEvents
	local remotes = TreeWalker.collectByClass("RemoteEvent")
	for _, remote in ipairs(remotes) do
		local remoteName = remote:GetFullName()
		trafficLog[remoteName] = { count = 0, totalSize = 0, timestamps = {} }

		local conn = (remote :: RemoteEvent).OnServerEvent:Connect(function(player, ...)
			if not monitoring then return end
			local entry = trafficLog[remoteName]
			if entry then
				entry.count += 1
				-- Estimate data size
				local args = { ... }
				local sizeEstimate = #HttpService:JSONEncode(args)
				entry.totalSize += sizeEstimate
				table.insert(entry.timestamps, os.clock() - startTime)
			end
		end)
		table.insert(connections, conn)
	end

	-- Monitor all RemoteFunctions
	local remoteFunctions = TreeWalker.collectByClass("RemoteFunction")
	for _, rf in ipairs(remoteFunctions) do
		local rfName = rf:GetFullName()
		trafficLog[rfName] = { count = 0, totalSize = 0, timestamps = {} }
		-- Note: Can't easily hook OnServerInvoke without replacing it
		-- We track the existence for the report
	end

	return true, {
		message = "Network monitoring started",
		trackingRemotes = #remotes,
		trackingFunctions = #remoteFunctions,
	}, nil
end

function NetworkMonitor.stop(args: { [string]: any }): (boolean, any, string?)
	if not monitoring then
		return false, nil, "Network monitor is not running"
	end

	monitoring = false
	local duration = os.clock() - startTime

	-- Disconnect all monitoring connections
	for _, conn in ipairs(connections) do
		conn:Disconnect()
	end
	connections = {}

	-- Build report
	local report: { any } = {}
	local totalCalls = 0
	local totalBytes = 0
	local spamDetected: { any } = {}

	for remoteName, data in pairs(trafficLog) do
		totalCalls += data.count
		totalBytes += data.totalSize

		local callsPerSecond = if duration > 0 then data.count / duration else 0
		local avgSize = if data.count > 0 then data.totalSize / data.count else 0

		local entry = {
			remote = remoteName,
			totalCalls = data.count,
			callsPerSecond = math.floor(callsPerSecond * 100) / 100,
			totalBytes = data.totalSize,
			avgBytesPerCall = math.floor(avgSize),
		}
		table.insert(report, entry)

		-- Spam detection: more than 10 calls per second is suspicious
		if callsPerSecond > 10 then
			table.insert(spamDetected, {
				remote = remoteName,
				callsPerSecond = callsPerSecond,
				severity = if callsPerSecond > 60 then "Critical" else "Warning",
			})
		end
	end

	-- Sort by call count
	table.sort(report, function(a, b) return a.totalCalls > b.totalCalls end)

	return true, {
		duration = math.floor(duration * 100) / 100,
		totalCalls = totalCalls,
		totalBytes = totalBytes,
		estimatedBandwidth = math.floor(totalBytes / math.max(duration, 1)) .. " bytes/sec",
		spamDetected = spamDetected,
		remotes = report,
	}, nil
end

return NetworkMonitor
