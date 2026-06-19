--!strict
local PluginState = require(script.Parent.Parent.Utils.PluginState)

-- Keep _G sync for backwards compatibility with server/client contexts
_G.StudioLinkMode = _G.StudioLinkMode or "stop"

return function(args: { [string]: any }): (boolean, any, string?)
	local mode = args.mode
	if not mode then
		return false, nil, "mode is required (start_play, stop, or run_server)"
	end

	-- Lazy lookup: StudioTestService may not be available at module load time
	local StudioTestService = game:FindFirstChildOfClass("StudioTestService") :: any
	if not StudioTestService then
		return false, nil, "StudioTestService not available"
	end

	if mode == "start_play" then
		PluginState.mode = "start_play"
		_G.StudioLinkMode = "start_play"
		-- Clear any stale stop request
		local pluginRef = PluginState.plugin or _G.StudioLinkPlugin
		if pluginRef then
			pcall(function() pluginRef:SetSetting("StudioLink_StopRequested", false) end)
		end
		-- Use task.defer so response is sent BEFORE play mode starts
		task.defer(function()
			-- ExecutePlayModeAsync yields until play ends. Without pcall, a throw
			-- (already in play, invalid options) would kill this thread and leave
			-- mode stuck at "start_play" forever — run_script_in_play_mode and the
			-- stop listener would then misbehave for the rest of the session.
			local ok, err = pcall(function()
				StudioTestService:ExecutePlayModeAsync({})
			end)
			if not ok then
				warn("[StudioLink] ExecutePlayModeAsync failed: " .. tostring(err))
			end
			PluginState.mode = "stop"
			_G.StudioLinkMode = "stop"
		end)
		return true, "Play mode started", nil

	elseif mode == "stop" then
		PluginState.mode = "stop"
		_G.StudioLinkMode = "stop"
		-- Signal the Server context stop-listener via plugin settings
		local pluginRef = PluginState.plugin or _G.StudioLinkPlugin
		if pluginRef then
			local ok, err = pcall(function()
				pluginRef:SetSetting("StudioLink_StopRequested", true)
			end)
			if ok then
				return true, "Stopped", nil
			else
				return false, nil, "Failed to signal stop: " .. tostring(err)
			end
		else
			return false, nil, "Plugin reference not available"
		end

	elseif mode == "run_server" then
		PluginState.mode = "run_server"
		_G.StudioLinkMode = "run_server"
		local pluginRef = PluginState.plugin or _G.StudioLinkPlugin
		if pluginRef then
			pcall(function() pluginRef:SetSetting("StudioLink_StopRequested", false) end)
		end
		task.defer(function()
			-- Same guard as start_play: a throw here must not strand mode at
			-- "run_server".
			local ok, err = pcall(function()
				StudioTestService:ExecuteRunModeAsync({})
			end)
			if not ok then
				warn("[StudioLink] ExecuteRunModeAsync failed: " .. tostring(err))
			end
			PluginState.mode = "stop"
			_G.StudioLinkMode = "stop"
		end)
		return true, "Server started (Run mode)", nil

	else
		return false, nil, "Invalid mode: " .. mode .. ". Use start_play, stop, or run_server"
	end
end
