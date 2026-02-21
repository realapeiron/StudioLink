--!strict
local RunService = game:GetService("RunService")

return function(args: { [string]: any }): (boolean, any, string?)
	-- Check RunService state (works in Server/Client context during play)
	if RunService:IsRunning() then
		if RunService:IsClient() then
			return true, "start_play", nil
		else
			return true, "run_server", nil
		end
	end

	-- In Edit context, RunService can't detect play mode
	-- Use the global mode flag set by StartStopPlay tool
	local trackedMode = _G.StudioLinkMode
	if trackedMode and trackedMode ~= "stop" then
		return true, trackedMode, nil
	end

	return true, "stop", nil
end
