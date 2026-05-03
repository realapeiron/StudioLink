--!strict
type Bucket = { count: number, paths: { string }, total_seconds: number }

return function(_args: { [string]: any }): (boolean, any, string?)
	local roots = {
		game.Workspace,
		game:GetService("ReplicatedStorage"),
		game:GetService("ServerStorage"),
		game:GetService("StarterGui"),
		game:GetService("StarterPlayer"),
	}

	local meshes: { [string]: Bucket } = {}
	local textures: { [string]: Bucket } = {}
	local sounds: { [string]: Bucket } = {}
	local animations: { [string]: Bucket } = {}

	local function record(
		map: { [string]: Bucket },
		id: string?,
		instance: Instance,
		seconds: number?
	)
		if not id or id == "" or id == "rbxassetid://0" or id == "rbxasset://" then
			return
		end
		local key = id :: string
		local bucket = map[key]
		if not bucket then
			bucket = { count = 0, paths = {}, total_seconds = 0 }
			map[key] = bucket
		end
		bucket.count = bucket.count + 1
		if #bucket.paths < 10 then
			table.insert(bucket.paths, instance:GetFullName())
		end
		if seconds and seconds > 0 then
			bucket.total_seconds = bucket.total_seconds + seconds
		end
	end

	for _, root in ipairs(roots) do
		local ok, descendants = pcall(function()
			return root:GetDescendants()
		end)
		if not ok then
			continue
		end
		for _, descendant in ipairs(descendants) do
			if descendant:IsA("MeshPart") then
				record(meshes, (descendant :: any).MeshId, descendant)
			elseif descendant:IsA("SpecialMesh") then
				record(meshes, (descendant :: any).MeshId, descendant)
			elseif descendant:IsA("Decal") or descendant:IsA("Texture") then
				record(textures, (descendant :: any).Texture, descendant)
			elseif descendant:IsA("ImageLabel") or descendant:IsA("ImageButton") then
				record(textures, (descendant :: any).Image, descendant)
			elseif descendant:IsA("Sound") then
				record(sounds, descendant.SoundId, descendant, descendant.TimeLength)
			elseif descendant:IsA("Animation") then
				record(animations, descendant.AnimationId, descendant)
			end
		end
	end

	local function flatten(map: { [string]: Bucket }): { any }
		local result = {}
		for id, data in pairs(map) do
			local entry: any = {
				asset_id = id,
				count = data.count,
				paths = data.paths,
			}
			if data.total_seconds > 0 then
				entry.total_seconds = data.total_seconds
			end
			table.insert(result, entry)
		end
		table.sort(result, function(a, b)
			return a.count > b.count
		end)
		return result
	end

	local meshList = flatten(meshes)
	local textureList = flatten(textures)
	local soundList = flatten(sounds)
	local animList = flatten(animations)

	return true,
		{
			meshes = meshList,
			textures = textureList,
			sounds = soundList,
			animations = animList,
			summary = {
				unique_meshes = #meshList,
				unique_textures = #textureList,
				unique_sounds = #soundList,
				unique_animations = #animList,
			},
			note = "Per-asset byte size is not exposed by Roblox plugin APIs. "
				.. "Use 'count' (reuse weight) and 'total_seconds' (audio/animation length) "
				.. "as proxies. For actual file sizes, see Studio's Asset Manager.",
		},
		nil
end
