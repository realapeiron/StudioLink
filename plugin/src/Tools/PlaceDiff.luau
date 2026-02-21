--!strict
-- PlaceDiff: Snapshot and diff place state

local TreeWalker = require(script.Parent.Parent.Utils.TreeWalker)
local HttpService = game:GetService("HttpService")

local PlaceDiff = {}

-- In-memory snapshot storage
local snapshots: { [string]: any } = {}

function PlaceDiff.take(args: { [string]: any }): (boolean, any, string?)
	local name = args.name or ("snapshot_" .. os.time())

	local services = {
		game:GetService("Workspace"),
		game:GetService("ServerScriptService"),
		game:GetService("ServerStorage"),
		game:GetService("ReplicatedStorage"),
		game:GetService("ReplicatedFirst"),
		game:GetService("StarterGui"),
		game:GetService("StarterPack"),
		game:GetService("StarterPlayer"),
		game:GetService("Lighting"),
	}

	local tree = {}
	for _, svc in ipairs(services) do
		table.insert(tree, TreeWalker.serializeTree(svc, 30))
	end

	snapshots[name] = {
		name = name,
		timestamp = os.time(),
		tree = tree,
	}

	return true, {
		message = "Snapshot '" .. name .. "' saved",
		name = name,
		timestamp = os.time(),
	}, nil
end

-- Flatten a tree into a path→data map for comparison
local function flattenTree(tree: { any }, prefix: string?): { [string]: any }
	local flat: { [string]: any } = {}
	prefix = prefix or ""

	for _, node in ipairs(tree) do
		local path = prefix .. "/" .. (node.Name or "?")
		flat[path] = {
			ClassName = node.ClassName,
			Source = node.Source,
			Position = node.Position,
			Size = node.Size,
			Attributes = node.Attributes,
		}
		if node.Children then
			local childFlat = flattenTree(node.Children, path)
			for k, v in pairs(childFlat) do
				flat[k] = v
			end
		end
	end

	return flat
end

function PlaceDiff.compare(args: { [string]: any }): (boolean, any, string?)
	local nameA = args.snapshotA
	local nameB = args.snapshotB

	if not snapshots[nameA] then
		return false, nil, "Snapshot '" .. nameA .. "' not found"
	end
	if not snapshots[nameB] then
		return false, nil, "Snapshot '" .. nameB .. "' not found"
	end

	local flatA = flattenTree(snapshots[nameA].tree)
	local flatB = flattenTree(snapshots[nameB].tree)

	local added: { string } = {}
	local removed: { string } = {}
	local changed: { any } = {}

	-- Find added and changed
	for path, dataB in pairs(flatB) do
		if not flatA[path] then
			table.insert(added, path)
		else
			local dataA = flatA[path]
			local diffs: { string } = {}

			if dataA.Source ~= dataB.Source and (dataA.Source or dataB.Source) then
				table.insert(diffs, "Source changed")
			end
			if dataA.Position ~= dataB.Position and (dataA.Position or dataB.Position) then
				table.insert(diffs, "Position: " .. tostring(dataA.Position) .. " → " .. tostring(dataB.Position))
			end
			if dataA.Size ~= dataB.Size and (dataA.Size or dataB.Size) then
				table.insert(diffs, "Size: " .. tostring(dataA.Size) .. " → " .. tostring(dataB.Size))
			end

			if #diffs > 0 then
				table.insert(changed, { path = path, changes = diffs })
			end
		end
	end

	-- Find removed
	for path in pairs(flatA) do
		if not flatB[path] then
			table.insert(removed, path)
		end
	end

	return true, {
		snapshotA = nameA,
		snapshotB = nameB,
		added = added,
		removed = removed,
		changed = changed,
		summary = {
			addedCount = #added,
			removedCount = #removed,
			changedCount = #changed,
		},
	}, nil
end

function PlaceDiff.list(args: { [string]: any }?): (boolean, any, string?)
	local list = {}
	for name, data in pairs(snapshots) do
		table.insert(list, {
			name = name,
			timestamp = data.timestamp,
		})
	end
	return true, list, nil
end

return PlaceDiff
