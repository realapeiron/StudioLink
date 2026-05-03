--!strict
local PlayHelpers = require(script.Parent.Parent.Utils.PlayHelpers)

local VALID_ACTIONS: { [string]: boolean } = {
	jump = true,
	sit = true,
	unsit = true,
	set_walkspeed = true,
	set_jumppower = true,
	set_health = true,
	heal = true,
	kill = true,
}

return function(args: { [string]: any }): (boolean, any, string?)
	local ctxOk, ctxErr = PlayHelpers.requireContext("play")
	if not ctxOk then
		return false, nil, ctxErr
	end

	local action = args.action
	if type(action) ~= "string" or not VALID_ACTIONS[action] then
		return false,
			nil,
			"action must be one of: jump, sit, unsit, set_walkspeed, set_jumppower, set_health, heal, kill"
	end

	local player, perr = PlayHelpers.resolvePlayer(args.player)
	if not player then
		return false, nil, perr
	end

	local character = player.Character
	if not character then
		return false, nil, "player '" .. player.Name .. "' has no character spawned"
	end

	local humanoid = character:FindFirstChildOfClass("Humanoid")
	if not humanoid then
		return false, nil, "no Humanoid in character"
	end

	local appliedValue: any = nil

	if action == "jump" then
		humanoid.Jump = true
	elseif action == "sit" then
		humanoid.Sit = true
	elseif action == "unsit" then
		humanoid.Sit = false
	elseif action == "set_walkspeed" then
		local v = args.value
		if type(v) ~= "number" or v < 0 then
			return false, nil, "value must be a non-negative number"
		end
		humanoid.WalkSpeed = v
		appliedValue = v
	elseif action == "set_jumppower" then
		local v = args.value
		if type(v) ~= "number" or v < 0 then
			return false, nil, "value must be a non-negative number"
		end
		humanoid.JumpPower = v
		appliedValue = v
	elseif action == "set_health" then
		local v = args.value
		if type(v) ~= "number" then
			return false, nil, "value must be a number"
		end
		humanoid.Health = math.max(0, math.min(humanoid.MaxHealth, v))
		appliedValue = humanoid.Health
	elseif action == "heal" then
		humanoid.Health = humanoid.MaxHealth
		appliedValue = humanoid.MaxHealth
	elseif action == "kill" then
		humanoid.Health = 0
	end

	return true,
		{
			action = action,
			applied_value = appliedValue,
			current_health = humanoid.Health,
			max_health = humanoid.MaxHealth,
		},
		nil
end
