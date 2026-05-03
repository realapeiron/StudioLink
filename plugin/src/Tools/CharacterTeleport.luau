--!strict
local PlayHelpers = require(script.Parent.Parent.Utils.PlayHelpers)

return function(args: { [string]: any }): (boolean, any, string?)
	local ctxOk, ctxErr = PlayHelpers.requireContext("play")
	if not ctxOk then
		return false, nil, ctxErr
	end

	local target = args.target
	if type(target) ~= "table" or (#target ~= 3 and #target ~= 6) then
		return false, nil, "target must be [x,y,z] or [x,y,z, lookX,lookY,lookZ]"
	end

	local player, perr = PlayHelpers.resolvePlayer(args.player)
	if not player then
		return false, nil, perr
	end

	local character = player.Character
	if not character then
		return false, nil, "player '" .. player.Name .. "' has no character spawned"
	end

	local pos = Vector3.new(target[1], target[2], target[3])
	local cf
	if #target == 6 then
		cf = CFrame.lookAt(pos, Vector3.new(target[4], target[5], target[6]))
	else
		cf = CFrame.new(pos)
	end

	if args.anchor_during then
		local hrp = character:FindFirstChild("HumanoidRootPart") :: BasePart?
		if hrp then
			hrp.Anchored = true
			character:PivotTo(cf)
			task.wait(0.05)
			hrp.Anchored = false
		else
			character:PivotTo(cf)
		end
	else
		character:PivotTo(cf)
	end

	return true, { new_position = { pos.X, pos.Y, pos.Z } }, nil
end
