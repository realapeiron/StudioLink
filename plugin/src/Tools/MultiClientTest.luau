--!strict
local PluginState = require(script.Parent.Parent.Utils.PluginState)

return function(args: { [string]: any }): (boolean, any, string?)
	local numPlayers = args.numPlayers or args.num_players or 2
	if type(numPlayers) ~= "number" or numPlayers < 1 or numPlayers > 8 then
		return false, nil, "numPlayers must be between 1 and 8"
	end

	local StudioTestService = game:FindFirstChildOfClass("StudioTestService") :: any
	if not StudioTestService then
		return false, nil, "StudioTestService not available"
	end

	PluginState.mode = "start_play"
	_G.StudioLinkMode = "start_play"

	local pluginRef = PluginState.plugin or _G.StudioLinkPlugin
	if pluginRef then
		pcall(function()
			pluginRef:SetSetting("StudioLink_StopRequested", false)
		end)
	end

	-- task.defer so the HTTP response goes back BEFORE play mode starts
	-- (ExecutePlayModeAsync yields until the play session ends).
	task.defer(function()
		-- Try the documented option name first; if Roblox rejects it,
		-- fall back to {numClients=N}, then to an empty options table.
		local ok = pcall(function()
			StudioTestService:ExecutePlayModeAsync({ numberOfPlayers = numPlayers })
		end)
		if not ok then
			ok = pcall(function()
				StudioTestService:ExecutePlayModeAsync({ numClients = numPlayers })
			end)
		end
		if not ok then
			pcall(function()
				StudioTestService:ExecutePlayModeAsync({})
			end)
		end
		PluginState.mode = "stop"
		_G.StudioLinkMode = "stop"
	end)

	return true,
		{
			started = true,
			num_players_requested = numPlayers,
			hint = "Use list_sessions to see registered server + client sessions once they connect.",
		},
		nil
end
