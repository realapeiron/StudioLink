--!strict
-- DataStoreDebug: DataStore inspection and manipulation

local DataStoreService = game:GetService("DataStoreService")
local HttpService = game:GetService("HttpService")

local DataStoreDebug = {}

function DataStoreDebug.list(args: { [string]: any }): (boolean, any, string?)
	local ok, result = pcall(function()
		local stores = DataStoreService:ListDataStoresAsync()
		local names: { string } = {}
		while true do
			local page = stores:GetCurrentPage()
			for _, info in ipairs(page) do
				table.insert(names, info.DataStoreName)
			end
			if stores.IsFinished then break end
			stores:AdvanceToNextPageAsync()
		end
		return names
	end)

	if ok then
		return true, { stores = result, count = #result }, nil
	else
		return false, nil, "Failed to list DataStores: " .. tostring(result)
			.. "\nMake sure 'Allow Studio Access to API Services' is enabled in Game Settings > Security"
	end
end

function DataStoreDebug.get(args: { [string]: any }): (boolean, any, string?)
	local storeName = args.storeName
	local key = args.key

	if not storeName or not key then
		return false, nil, "storeName and key are required"
	end

	local ok, result = pcall(function()
		local store = DataStoreService:GetDataStore(storeName)
		return store:GetAsync(key)
	end)

	if ok then
		return true, { storeName = storeName, key = key, value = result }, nil
	else
		return false, nil, "Failed to get key: " .. tostring(result)
	end
end

function DataStoreDebug.set(args: { [string]: any }): (boolean, any, string?)
	local storeName = args.storeName
	local key = args.key
	local value = args.value

	if not storeName or not key then
		return false, nil, "storeName and key are required"
	end

	local ok, result = pcall(function()
		local store = DataStoreService:GetDataStore(storeName)
		store:SetAsync(key, value)
		return true
	end)

	if ok then
		return true, { message = "Key '" .. key .. "' updated in '" .. storeName .. "'" }, nil
	else
		return false, nil, "Failed to set key: " .. tostring(result)
	end
end

function DataStoreDebug.delete(args: { [string]: any }): (boolean, any, string?)
	local storeName = args.storeName
	local key = args.key

	if not storeName or not key then
		return false, nil, "storeName and key are required"
	end

	local ok, result = pcall(function()
		local store = DataStoreService:GetDataStore(storeName)
		store:RemoveAsync(key)
		return true
	end)

	if ok then
		return true, { message = "Key '" .. key .. "' deleted from '" .. storeName .. "'" }, nil
	else
		return false, nil, "Failed to delete key: " .. tostring(result)
	end
end

function DataStoreDebug.scan(args: { [string]: any }): (boolean, any, string?)
	local storeName = args.storeName
	local pageSize = args.pageSize or 50

	if not storeName then
		return false, nil, "storeName is required"
	end

	local ok, result = pcall(function()
		local store = DataStoreService:GetDataStore(storeName)
		local pages = store:ListKeysAsync("", pageSize)
		local keys: { any } = {}

		local page = pages:GetCurrentPage()
		for _, keyInfo in ipairs(page) do
			table.insert(keys, {
				key = keyInfo.KeyName,
			})
		end

		return {
			keys = keys,
			count = #keys,
			hasMore = not pages.IsFinished,
		}
	end)

	if ok then
		return true, result, nil
	else
		return false, nil, "Failed to scan DataStore: " .. tostring(result)
	end
end

return DataStoreDebug
