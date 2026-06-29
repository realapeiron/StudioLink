--!strict
-- viewport_capture — grab the Studio viewport entirely in-engine (no OS screen-
-- recording permission). CaptureService:CaptureScreenshot → EditableImage →
-- tiled ReadPixelsBuffer → base64 RGBA. The Rust server decodes and PNG-encodes.
-- Works in Edit mode with the viewport visible; needs Game Settings > Security >
-- "Allow Mesh / Image APIs".

local CaptureService = game:GetService("CaptureService")
local AssetService = game:GetService("AssetService")
local Base64 = require(script.Parent.Parent.Utils.Base64)

local READ_TILE = 1024 -- ReadPixelsBuffer caps per-call pixel area; tile large viewports.

return function(args: { [string]: any }): (boolean, any, string?)
	local done, cid = false, nil
	CaptureService:CaptureScreenshot(function(contentId)
		cid = contentId
		done = true
	end)
	local startTime = os.clock()
	while not done and os.clock() - startTime < 10 do
		task.wait(0.1)
	end
	if not cid then
		return false,
			nil,
			"Screenshot capture timed out (10s). Make sure the Studio viewport is visible and you are in Edit mode. "
				.. "Known Roblox quirk: capture can fail if the viewport renders a solid color."
	end

	local img: any = nil
	pcall(function()
		img = AssetService:CreateEditableImageAsync(Content.fromUri(cid))
	end)
	if not img then
		return false,
			nil,
			"CreateEditableImageAsync failed — enable Game Settings > Security > 'Allow Mesh / Image APIs'."
	end

	local w, h = img.Size.X, img.Size.Y
	local full = buffer.create(w * h * 4)
	local readOk, readErr = pcall(function()
		local ty = 0
		while ty < h do
			local th = math.min(READ_TILE, h - ty)
			local tx = 0
			while tx < w do
				local tw = math.min(READ_TILE, w - tx)
				local tile = img:ReadPixelsBuffer(Vector2.new(tx, ty), Vector2.new(tw, th))
				for row = 0, th - 1 do
					buffer.copy(full, ((ty + row) * w + tx) * 4, tile, row * tw * 4, tw * 4)
				end
				tx += READ_TILE
			end
			ty += READ_TILE
		end
	end)
	pcall(function()
		img:Destroy()
	end)
	if not readOk then
		return false, nil, "ReadPixelsBuffer failed: " .. tostring(readErr)
	end

	return true, { width = w, height = h, data = Base64.encode(full) }, nil
end
