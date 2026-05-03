--!strict
local PlayHelpers = require(script.Parent.Parent.Utils.PlayHelpers)

return function(args: { [string]: any }): (boolean, any, string?)
	local ctxOk, ctxErr = PlayHelpers.requireContext("play")
	if not ctxOk then
		return false, nil, ctxErr
	end

	local target = args.target
	if type(target) ~= "table" or #target < 3 then
		return false, nil, "target must be an [x, y, z] array"
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

	local pos = Vector3.new(target[1], target[2], target[3])
	local startTime = os.clock()
	humanoid:MoveTo(pos)

	local waitFinished: boolean = if args.wait_finished == nil then true else args.wait_finished
	local timeoutSecs: number = if type(args.timeout_secs) == "number" then args.timeout_secs else 8

	local reached = false
	if waitFinished then
		local done = false
		local conn = humanoid.MoveToFinished:Connect(function(success: boolean)
			reached = success
			done = true
		end)
		local deadline = os.clock() + timeoutSecs
		while not done and os.clock() < deadline do
			task.wait(0.05)
		end
		conn:Disconnect()
	end

	local hrp = character:FindFirstChild("HumanoidRootPart") :: BasePart?
	local currentPos = if hrp then hrp.Position else pos
	local distRemaining = (currentPos - pos).Magnitude

	return true,
		{
			reached = reached,
			distance_remaining = distRemaining,
			duration_ms = math.floor((os.clock() - startTime) * 1000),
			final_position = { currentPos.X, currentPos.Y, currentPos.Z },
			waited = waitFinished,
		},
		nil
end
