--!strict
local InsertService = game:GetService("InsertService")
local MarketplaceService = game:GetService("MarketplaceService")

return function(args: { [string]: any }): (boolean, any, string?)
	local query = args.query
	if not query or query == "" then
		return false, nil, "No search query provided"
	end

	local modelId: number? = nil
	local modelName: string = "InsertedModel"

	-- Check if query is a numeric asset ID
	local numericId = tonumber(query)
	if numericId then
		modelId = numericId
		local nameOk, info = pcall(function()
			return MarketplaceService:GetProductInfo(numericId)
		end)
		if nameOk and info then
			modelName = info.Name or modelName
		end
	else
		-- Search via InsertService:GetFreeModels (returns pages format)
		local searchOk, pages = pcall(function()
			return InsertService:GetFreeModels(query, 0)
		end)

		if searchOk and pages and #pages > 0 then
			local page = pages[1]
			if page.Results and #page.Results > 0 then
				local firstResult = page.Results[1]
				modelId = firstResult.AssetId
				modelName = firstResult.Name or modelName
			end
		end

		if not modelId then
			return false, nil, "No models found for query: " .. query .. ". Try different keywords or provide a direct asset ID."
		end
	end

	-- Load and insert the model
	local loadOk, loadedModel = pcall(function()
		return InsertService:LoadAsset(modelId :: number)
	end)

	if not loadOk or not loadedModel then
		return false, nil, "Failed to load model ID " .. tostring(modelId) .. ": " .. tostring(loadedModel)
	end

	-- LoadAsset returns a Model container — move children to Workspace
	local insertedNames: { string } = {}
	for _, child in ipairs(loadedModel:GetChildren()) do
		child.Parent = game:GetService("Workspace")
		table.insert(insertedNames, child.Name)
	end
	loadedModel:Destroy()

	return true, {
		message = "Inserted model: " .. modelName .. " (ID: " .. tostring(modelId) .. ")",
		modelId = modelId,
		modelName = modelName,
		insertedObjects = insertedNames,
	}, nil
end
