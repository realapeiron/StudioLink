--!strict
-- input_simulate — drive Studio's keyboard/mouse via VirtualInputManager.
-- Strategy:
--   "vim"       → SendKeyEvent / SendMouseButtonEvent direct.
--   "injection" → LocalScript bridge (NOT YET IMPLEMENTED).
--   "auto"      → tries vim; falls back to a clear error if blocked.
-- Run vim_capability_test first to confirm VirtualInputManager access.

local function getKeyCode(name: any): EnumItem?
	if type(name) ~= "string" then
		return nil
	end
	local ok, kc = pcall(function()
		return (Enum.KeyCode :: any)[name]
	end)
	if ok and kc then
		return kc
	end
	return nil
end

local function buttonId(name: any): number
	if name == "Right" then
		return 1
	elseif name == "Middle" then
		return 2
	end
	return 0 -- Left default
end

local function execAction(vim: any, action: any): (boolean, string?)
	local actionType = action.type
	if actionType == "key" then
		local kc = getKeyCode(action.key)
		if not kc then
			return false, "unknown key: " .. tostring(action.key)
		end
		local mode = action.mode or "tap"
		local ok, err = pcall(function()
			if mode == "press" or mode == "tap" then
				vim:SendKeyEvent(true, kc, false, game)
			end
			if mode == "tap" then
				task.wait((action.duration_ms or 50) / 1000)
			end
			if mode == "release" or mode == "tap" then
				vim:SendKeyEvent(false, kc, false, game)
			end
		end)
		if ok then
			return true, nil
		end
		return false, "SendKeyEvent failed: " .. tostring(err)
	elseif actionType == "mouse_click" then
		local pos = action.position
		if type(pos) ~= "table" or #pos < 2 then
			return false, "position must be [x, y]"
		end
		local btn = buttonId(action.button)
		local ok, err = pcall(function()
			vim:SendMouseButtonEvent(pos[1], pos[2], btn, true, nil, 1)
			task.wait((action.hold_ms or 50) / 1000)
			vim:SendMouseButtonEvent(pos[1], pos[2], btn, false, nil, 1)
		end)
		if ok then
			return true, nil
		end
		return false, "SendMouseButtonEvent failed: " .. tostring(err)
	elseif actionType == "mouse_move" then
		local pos = action.position
		if type(pos) ~= "table" or #pos < 2 then
			return false, "position must be [x, y]"
		end
		local ok, err = pcall(function()
			vim:SendMouseMoveEvent(pos[1], pos[2], nil)
		end)
		if ok then
			return true, nil
		end
		return false, "SendMouseMoveEvent failed: " .. tostring(err)
	elseif actionType == "key_combo" then
		local keys = action.keys
		if type(keys) ~= "table" then
			return false, "keys must be an array of key names"
		end
		local kcs: { EnumItem } = {}
		for i, k in ipairs(keys) do
			local kc = getKeyCode(k)
			if not kc then
				return false, "unknown key in combo: " .. tostring(k)
			end
			kcs[i] = kc
		end
		local ok, err = pcall(function()
			for _, kc in ipairs(kcs) do
				vim:SendKeyEvent(true, kc, false, game)
			end
			task.wait((action.duration_ms or 50) / 1000)
			for i = #kcs, 1, -1 do
				vim:SendKeyEvent(false, kcs[i], false, game)
			end
		end)
		if ok then
			return true, nil
		end
		return false, "key_combo failed: " .. tostring(err)
	end
	return false, "unknown action type: " .. tostring(actionType)
end

return function(args: { [string]: any }): (boolean, any, string?)
	local actions = args.actions
	if type(actions) ~= "table" or #actions == 0 then
		return false, nil, "actions must be a non-empty array"
	end

	local strategy: string = args.strategy or "auto"
	if strategy ~= "vim" and strategy ~= "injection" and strategy ~= "auto" then
		return false, nil, "strategy must be 'vim', 'injection', or 'auto'"
	end
	if strategy == "injection" then
		return false,
			nil,
			"injection strategy is not yet implemented. Use vim_capability_test to verify direct access, then strategy='vim' or 'auto'."
	end

	local betweenDelayMs: number = if type(args.between_action_delay_ms) == "number"
		then args.between_action_delay_ms
		else 16

	local vim = game:GetService("VirtualInputManager") :: any
	if not vim then
		return false, nil, "VirtualInputManager service not found"
	end

	local executed = 0
	local errors: { { index: number, error: string } } = {}

	for i, action in ipairs(actions) do
		local ok, err = execAction(vim, action)
		if ok then
			executed = executed + 1
		else
			table.insert(errors, { index = i, error = err or "unknown error" })
		end
		if i < #actions then
			task.wait(betweenDelayMs / 1000)
		end
	end

	return true,
		{
			strategy_used = "vim",
			executed_count = executed,
			total_count = #actions,
			errors = if #errors > 0 then errors else nil,
		},
		nil
end
