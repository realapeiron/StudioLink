--!strict
-- DataStoreDebug: DataStore inspection and manipulation

local DataStoreService = game:GetService("DataStoreService")
local HttpService = game:GetService("HttpService")

local DataStoreDebug = {}

-- Roblox rate-limits DataStore calls; when the per-minute budget for a request
-- type hits zero the next call blocks/throttles. Check first and return a clear
-- message instead of a confusing deep failure. pcall-guarded so a missing API
-- never breaks the tool — if the budget can't be read, we let the call proceed.
local function ensureBudget(requestType: Enum.DataStoreRequestType): (boolean, string?)
	local ok, budget = pcall(function()
		return DataStoreService:GetRequestBudgetForRequestType(requestType)
	end)
	if ok and type(budget) == "number" and budget < 1 then
		return false,
			"DataStore request budget exhausted — wait a few seconds and retry (Roblox rate-limits DataStore calls)."
	end
	return true, nil
end

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

	local budgetOk, budgetErr = ensureBudget(Enum.DataStoreRequestType.GetAsync)
	if not budgetOk then
		return false, nil, budgetErr
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

	local budgetOk, budgetErr = ensureBudget(Enum.DataStoreRequestType.SetIncrementAsync)
	if not budgetOk then
		return false, nil, budgetErr
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

	local budgetOk, budgetErr = ensureBudget(Enum.DataStoreRequestType.SetIncrementAsync)
	if not budgetOk then
		return false, nil, budgetErr
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
	local maxPages = args.maxPages or 1

	if not storeName then
		return false, nil, "storeName is required"
	end

	local ok, result = pcall(function()
		local store = DataStoreService:GetDataStore(storeName)
		local pages = store:ListKeysAsync("", pageSize)
		local keys: { any } = {}
		local pageCount = 0

		repeat
			pageCount += 1
			local page = pages:GetCurrentPage()
			for _, keyInfo in ipairs(page) do
				table.insert(keys, {
					key = keyInfo.KeyName,
				})
			end
			if pages.IsFinished or pageCount >= maxPages then break end
			pages:AdvanceToNextPageAsync()
		until false

		return {
			keys = keys,
			count = #keys,
			hasMore = not pages.IsFinished,
			pagesScanned = pageCount,
		}
	end)

	if ok then
		return true, result, nil
	else
		return false, nil, "Failed to scan DataStore: " .. tostring(result)
	end
end

return DataStoreDebug
