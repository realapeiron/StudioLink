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
		return false,
			nil,
			"selector resolved to " .. instance.ClassName .. ", not a GuiButton"
	end

	-- GuiButton:Activate() fires the Activated event. Roblox added it in 2024.
	-- Server-side listeners trigger normally; client-side listeners run if
	-- the change is replicated to the client (Activated propagates).
	local activate = (instance :: any).Activate
	if type(activate) ~= "function" then
		return false,
			nil,
			"GuiButton:Activate() unavailable (older Studio?). Use input_simulate as a fallback."
	end

	local ok, err = pcall(function()
		(instance :: any):Activate()
	end)
	if not ok then
		return false, nil, "Activate() failed: " .. tostring(err)
	end

	return true,
		{
			target_path = instance:GetFullName(),
			method_used = "activate",
			note = "Activated event fired. Client-side listeners run via replication.",
		},
		nil
end
