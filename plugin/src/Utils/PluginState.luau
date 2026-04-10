--!strict
-- PluginState: Shared plugin state (replaces _G pollution)

local PluginState = {
	mode = "stop" :: string,
	plugin = nil :: any,
}

return PluginState
