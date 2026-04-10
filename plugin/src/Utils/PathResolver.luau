--!strict
-- PathResolver: Shared instance path resolution utility

local PathResolver = {}

-- Resolve a dot-separated path like "Workspace.Model.Part" to an Instance
function PathResolver.resolve(path: string): Instance?
	local parts = path:split(".")
	local current: Instance = game

	-- Handle "game" prefix
	local startIndex = 1
	if parts[1] == "game" then
		startIndex = 2
	end

	for i = startIndex, #parts do
		local child = current:FindFirstChild(parts[i])
		if not child then
			-- Try GetService for top-level services
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

return PathResolver
