--!strict
local PluginUtils = {}

function PluginUtils.getSetting(plugin: Plugin, key: string, default: any): any
	local ok, value = pcall(function()
		return plugin:GetSetting(key)
	end)
	if ok and value ~= nil then
		return value
	end
	return default
end

function PluginUtils.setSetting(plugin: Plugin, key: string, value: any)
	pcall(function()
		plugin:SetSetting(key, value)
	end)
end

return PluginUtils
