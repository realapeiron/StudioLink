--!strict
-- viewport_screenshot — trigger StudioService:TakeScreenshot(). The Rust
-- server polls the screenshot directory for the new file and base64-encodes
-- it. This tool fires from Edit context and returns immediately.

return function(_args: { [string]: any }): (boolean, any, string?)
	local StudioService = game:GetService("StudioService") :: any
	local ok, err = pcall(function()
		StudioService:TakeScreenshot()
	end)
	if not ok then
		return false, nil, "TakeScreenshot failed: " .. tostring(err)
	end
	return true,
		{
			triggered = true,
			capture_method = "StudioService.TakeScreenshot",
			note = "Server polls the Roblox Screenshots dir for the new file.",
		},
		nil
end
