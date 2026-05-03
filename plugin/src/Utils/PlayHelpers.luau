--!strict
-- PlayHelpers: shared helpers for in-game (play-mode) tools.
-- Player resolution, GUI/instance selectors, and context guards.

local Players = game:GetService("Players")
local CollectionService = game:GetService("CollectionService")
local RunService = game:GetService("RunService")
local PathResolver = require(script.Parent.PathResolver)

local PlayHelpers = {}

-- Resolve a player from a free-form spec.
--   nil or "@first" → Players:GetPlayers()[1]
--   string         → Players:FindFirstChild(name)
--   {userId = N}   → Players:GetPlayerByUserId(N)
function PlayHelpers.resolvePlayer(spec: any?): (Player?, string?)
	if spec == nil or spec == "@first" then
		local first = Players:GetPlayers()[1]
		if first then
			return first, nil
		end
		return nil, "no players in game (is play mode running and a player loaded?)"
	end

	if type(spec) == "string" then
		local p = Players:FindFirstChild(spec)
		if p and p:IsA("Player") then
			return p, nil
		end
		return nil, "player named '" .. spec .. "' not found"
	end

	if type(spec) == "table" and spec.userId then
		local ok, p = pcall(function()
			return Players:GetPlayerByUserId(spec.userId)
		end)
		if ok and p then
			return p, nil
		end
		return nil, "no player with userId " .. tostring(spec.userId)
	end

	return nil, "invalid player spec; use nil, '@first', a username, or {userId=N}"
end

-- Resolve a selector to an Instance.
--   string                          → treated as { path = string }
--   { path = "PlayerGui.Foo.Bar" }  → resolved relative to player when path starts with "PlayerGui"
--   { path = "Workspace.X" }        → PathResolver
--   { tag = "..." }                 → CollectionService:GetTagged (first match)
--   { attribute = {key, value} }    → first descendant whose attribute matches
function PlayHelpers.resolveSelector(spec: any, player: Player?): (Instance?, string?)
	if type(spec) == "string" then
		spec = { path = spec }
	end
	if type(spec) ~= "table" then
		return nil, "selector must be a string path or table"
	end

	if spec.path then
		local path: string = spec.path
		if player and path:sub(1, 9) == "PlayerGui" then
			local parts = path:split(".")
			local current: Instance = player
			for _, part in ipairs(parts) do
				local child = current:FindFirstChild(part)
				if not child then
					return nil, "selector path '" .. path .. "' broke at '" .. part .. "'"
				end
				current = child
			end
			return current, nil
		else
			local resolved = PathResolver.resolve(path)
			if resolved then
				return resolved, nil
			end
			return nil, "path '" .. path .. "' did not resolve"
		end
	end

	if spec.tag then
		local tagged = CollectionService:GetTagged(spec.tag)
		if #tagged == 0 then
			return nil, "no instances with tag '" .. tostring(spec.tag) .. "'"
		end
		return tagged[1], nil
	end

	if spec.attribute then
		local key = spec.attribute.key
		local targetValue = spec.attribute.value
		if not key then
			return nil, "attribute selector requires {key, value}"
		end
		local roots: { Instance } = {
			workspace,
			game:GetService("ReplicatedStorage"),
			game:GetService("ServerStorage"),
		}
		if player then
			table.insert(roots, player)
		end
		for _, root in ipairs(roots) do
			for _, descendant in ipairs(root:GetDescendants()) do
				if descendant:GetAttribute(key) == targetValue then
					return descendant, nil
				end
			end
		end
		return nil, "no instance with attribute " .. tostring(key) .. "=" .. tostring(targetValue)
	end

	return nil, "selector must have one of: path, tag, attribute"
end

-- Guard a tool against the wrong run context. needed ∈
-- {"edit", "play", "play_server", "play_client"}.
function PlayHelpers.requireContext(needed: string): (boolean, string?)
	local isRunning = RunService:IsRunning()
	local isServer = RunService:IsServer()
	local isClient = RunService:IsClient()
	local isEdit = RunService:IsEdit()

	if needed == "edit" then
		if isEdit then
			return true, nil
		end
		return false, "this tool requires Edit mode"
	elseif needed == "play_server" then
		if isRunning and isServer then
			return true, nil
		end
		return false, "this tool requires Play mode (server context)"
	elseif needed == "play_client" then
		if isRunning and isClient then
			return true, nil
		end
		return false, "this tool requires Play mode (client context)"
	elseif needed == "play" then
		if isRunning then
			return true, nil
		end
		return false, "this tool requires Play mode"
	end

	return false, "unknown context name: " .. needed
end

return PlayHelpers
