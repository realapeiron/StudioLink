--!strict
-- UndoRedo: Undo/Redo utilities using ChangeHistoryService for StudioLink

local UndoRedo = {}

function UndoRedo.undo(args: { [string]: any }): (boolean, any, string?)
	local CHS = game:GetService("ChangeHistoryService") :: any

	local ok, err = pcall(function()
		CHS:Undo()
	end)

	if not ok then
		return false, nil, "Failed to undo: " .. tostring(err)
	end

	return true, {
		action = "undo",
		success = true,
	}, nil
end

function UndoRedo.redo(args: { [string]: any }): (boolean, any, string?)
	local CHS = game:GetService("ChangeHistoryService") :: any

	local ok, err = pcall(function()
		CHS:Redo()
	end)

	if not ok then
		return false, nil, "Failed to redo: " .. tostring(err)
	end

	return true, {
		action = "redo",
		success = true,
	}, nil
end

return UndoRedo
