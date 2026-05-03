--!strict
local PlayHelpers = require(script.Parent.Parent.Utils.PlayHelpers)

return function(args: { [string]: any }): (boolean, any, string?)
	local ctxOk, ctxErr = PlayHelpers.requireContext("play")
	if not ctxOk then
		return false, nil, ctxErr
	end

	local player, perr = PlayHelpers.resolvePlayer(args.player)
	if not player then
		return false, nil, perr
	end

	local instance, serr = PlayHelpers.resolveSelector(args.selector, player)
	if not instance then
		return false, nil, serr
	end

	if not instance:IsA("GuiButton") then
		return false, nil, "selector resolved to " .. instance.ClassName .. ", not a GuiButton"
	end

	-- Best effort: if a Bindable-style hook is attached, fire it. Some games
	-- expose a BindableEvent named "_studiolink_click" alongside the button
	-- specifically so external tools can drive it; if found, we fire that.
	local hook = instance:FindFirstChild("_studiolink_click")
	if hook and hook:IsA("BindableEvent") then
		local ok, err = pcall(function()
			(hook :: BindableEvent):Fire(player)
		end)
		if ok then
			return true,
				{
					target_path = instance:GetFullName(),
					method_used = "bindable_hook",
					hook_name = "_studiolink_click",
				},
				nil
		end
		return false, nil, "BindableEvent fire failed: " .. tostring(err)
	end

	-- No hook present. Roblox does NOT expose a way to synthesize the
	-- built-in Activated/MouseButton1Click events from server context — those
	-- require real client input or RobloxScriptSecurity. Surface this clearly.
	return false,
		nil,
		"ui_click cannot synthesize a real GuiButton click from server context. "
			.. "Roblox built-in events (Activated, MouseButton1Click) cannot be fired manually. "
			.. "Workaround A: place a BindableEvent named '_studiolink_click' under the button — "
			.. "this tool will Fire() it with the player. "
			.. "Workaround B: use ui_set_text + read state, or implement a UIBridge LocalScript "
			.. "(planned for v0.6+) that listens on a RemoteEvent."
end
