--!strict
-- vim_capability_test — probe VirtualInputManager methods to determine which
-- ones are callable from this plugin context. Caches the result on
-- _G.StudioLink_VimReport so input_simulate can pick the right strategy.

return function(_args: { [string]: any }): (boolean, any, string?)
	local results: { [string]: { available: boolean, error: string? } } = {}

	local vim = game:GetService("VirtualInputManager") :: any
	if not vim then
		return true,
			{
				available = false,
				reason = "VirtualInputManager service not found",
				methods = results,
			},
			nil
	end

	-- Each probe uses the safest possible argument set. SendKeyEvent uses
	-- KeyCode.Unknown so even if it succeeds it shouldn't trigger any
	-- listeners.
	local probes: { { name: string, fn: () -> () } } = {
		{
			name = "SendKeyEvent",
			fn = function()
				vim:SendKeyEvent(true, Enum.KeyCode.Unknown, false, game)
			end,
		},
		{
			name = "SendMouseButtonEvent",
			fn = function()
				vim:SendMouseButtonEvent(0, 0, 0, true, nil, 1)
			end,
		},
		{
			name = "SendMouseMoveEvent",
			fn = function()
				vim:SendMouseMoveEvent(0, 0, nil)
			end,
		},
		{
			name = "SendMouseWheelEvent",
			fn = function()
				vim:SendMouseWheelEvent(0, 0, true, nil)
			end,
		},
	}

	for _, probe in ipairs(probes) do
		local ok, err = pcall(probe.fn)
		results[probe.name] = {
			available = ok,
			error = if ok then nil else tostring(err),
		}
	end

	-- Cache for input_simulate to consult later.
	_G.StudioLink_VimReport = results

	local RunService = game:GetService("RunService")
	return true,
		{
			methods = results,
			context = {
				is_edit = RunService:IsEdit(),
				is_running = RunService:IsRunning(),
				is_server = RunService:IsServer(),
				is_client = RunService:IsClient(),
			},
			note = "Run this in Edit mode AND during Play to compare. _G.StudioLink_VimReport is cached so input_simulate's auto strategy can fall back to injection when vim methods are blocked.",
		},
		nil
end
