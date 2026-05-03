--!strict
return function(args: { [string]: any }): (boolean, any, string?)
	local versionType = args.versionType or args.version_type or "Saved"
	if versionType ~= "Saved" and versionType ~= "Published" then
		return false, nil, "versionType must be 'Saved' or 'Published'"
	end

	local placeId = game.PlaceId or 0
	if placeId == 0 then
		return false,
			nil,
			"Cannot publish an unsaved local .rbxl. Save it first via File > Save to Roblox As..."
	end

	local StudioService = game:GetService("StudioService") :: any

	-- StudioService:PublishAs() opens Studio's publish dialog. The user must
	-- complete it manually — true headless publish requires RobloxScriptSecurity
	-- which plugins don't have. We surface this clearly in the response.
	local ok, err = pcall(function()
		StudioService:PublishAs()
	end)

	if not ok then
		return false,
			nil,
			"PublishAs failed (likely RobloxScriptSecurity restriction): "
				.. tostring(err)
				.. ". Use File > Publish to Roblox manually."
	end

	return true,
		{
			place_id = placeId,
			game_id = game.GameId or 0,
			version_type = versionType,
			dialog_opened = true,
			note = "Publish dialog opened in Studio. Complete it to finalize the upload.",
		},
		nil
end
