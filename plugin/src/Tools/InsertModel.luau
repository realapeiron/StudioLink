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
		-- Search via Toolbox API (GetFreeModels is deprecated and broken)
		local HttpService = game:GetService("HttpService")
		local searchOk, searchResult = pcall(function()
			local url = "https://apis.roblox.com/toolbox-service/v1/marketplace/"
				.. HttpService:UrlEncode(query)
				.. "?category=Models&limit=1&sortType=Relevance"
			local response = HttpService:RequestAsync({
				Url = url,
				Method = "GET",
			})
			if response.StatusCode == 200 then
				return HttpService:JSONDecode(response.Body)
			end
			return nil
		end)

		if searchOk and searchResult and searchResult.data and #searchResult.data > 0 then
			local firstResult = searchResult.data[1]
			modelId = firstResult.id or firstResult.assetId
			modelName = firstResult.name or firstResult.Name or modelName
		end

		if not modelId then
			return false, nil, "No models found for query: " .. query
				.. ". Try different keywords or provide a direct numeric asset ID."
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
