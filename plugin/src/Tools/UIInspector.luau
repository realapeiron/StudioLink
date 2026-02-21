--!strict
-- UIInspector: Analyze GUI hierarchy and detect issues

local TreeWalker = require(script.Parent.Parent.Utils.TreeWalker)

local UIInspector = {}

function UIInspector.tree(args: { [string]: any }): (boolean, any, string?)
	local starterGui = game:GetService("StarterGui")
	local guiTree: { any } = {}

	local function buildGuiTree(instance: Instance, depth: number): any?
		if depth > 20 then return nil end

		local node: { [string]: any } = {
			Name = instance.Name,
			ClassName = instance.ClassName,
		}

		if instance:IsA("GuiObject") then
			local guiObj = instance :: GuiObject
			node.Size = tostring(guiObj.Size)
			node.Position = tostring(guiObj.Position)
			node.Visible = guiObj.Visible
			node.ZIndex = guiObj.ZIndex
			if instance:IsA("TextLabel") or instance:IsA("TextButton") or instance:IsA("TextBox") then
				node.Text = (instance :: any).Text
			end
		end

		local children = {}
		for _, child in ipairs(instance:GetChildren()) do
			local childNode = buildGuiTree(child, depth + 1)
			if childNode then
				table.insert(children, childNode)
			end
		end

		if #children > 0 then
			node.Children = children
		end

		return node
	end

	for _, screenGui in ipairs(starterGui:GetChildren()) do
		local tree = buildGuiTree(screenGui, 0)
		if tree then
			table.insert(guiTree, tree)
		end
	end

	return true, guiTree, nil
end

function UIInspector.analyze(args: { [string]: any }): (boolean, any, string?)
	local issues: { any } = {}
	local starterGui = game:GetService("StarterGui")

	-- Collect all GuiObjects
	local allGui: { GuiObject } = {}
	TreeWalker.walkDescendants(starterGui, function(instance)
		if instance:IsA("GuiObject") then
			table.insert(allGui, instance :: GuiObject)
		end
	end)

	for _, gui in ipairs(allGui) do
		-- 1. Mobile touch target check (minimum 44x44 pixels recommended)
		if gui:IsA("GuiButton") or gui:IsA("TextBox") then
			local absSize = gui.AbsoluteSize
			if absSize.X < 44 or absSize.Y < 44 then
				table.insert(issues, {
					type = "Mobile Touch Target",
					severity = "Medium",
					location = gui:GetFullName(),
					description = string.format(
						"Button too small for mobile (%.0fx%.0f). Minimum recommended: 44x44px",
						absSize.X, absSize.Y
					),
				})
			end
		end

		-- 2. Off-screen elements
		local absPos = gui.AbsolutePosition
		local absSize = gui.AbsoluteSize
		if gui.Visible and (absPos.X + absSize.X < 0 or absPos.Y + absSize.Y < 0) then
			table.insert(issues, {
				type = "Off-Screen Element",
				severity = "Low",
				location = gui:GetFullName(),
				description = "Element is positioned off-screen",
			})
		end

		-- 3. ZIndex conflicts (same parent, overlapping, same ZIndex)
		if gui.Parent and gui.Parent:IsA("GuiObject") then
			for _, sibling in ipairs(gui.Parent:GetChildren()) do
				if sibling ~= gui and sibling:IsA("GuiObject") then
					local sibGui = sibling :: GuiObject
					if sibGui.ZIndex == gui.ZIndex and sibGui.Visible and gui.Visible then
						-- Check overlap
						local aPos, aSize = gui.AbsolutePosition, gui.AbsoluteSize
						local bPos, bSize = sibGui.AbsolutePosition, sibGui.AbsoluteSize

						local overlaps = aPos.X < bPos.X + bSize.X
							and aPos.X + aSize.X > bPos.X
							and aPos.Y < bPos.Y + bSize.Y
							and aPos.Y + aSize.Y > bPos.Y

						if overlaps then
							table.insert(issues, {
								type = "ZIndex Conflict",
								severity = "Low",
								location = gui:GetFullName(),
								description = "Overlaps with '" .. sibling.Name .. "' at same ZIndex",
							})
						end
					end
				end
			end
		end

		-- 4. Missing layout components (Frame with multiple children but no UIListLayout)
		if gui:IsA("Frame") or gui:IsA("ScrollingFrame") then
			local childCount = 0
			local hasLayout = false
			for _, child in ipairs(gui:GetChildren()) do
				if child:IsA("GuiObject") then childCount += 1 end
				if child:IsA("UIListLayout") or child:IsA("UIGridLayout") or child:IsA("UITableLayout") then
					hasLayout = true
				end
			end
			if childCount > 3 and not hasLayout then
				table.insert(issues, {
					type = "Missing Layout",
					severity = "Info",
					location = gui:GetFullName(),
					description = childCount .. " children without UIListLayout/UIGridLayout — consider adding a layout component",
				})
			end
		end
	end

	return true, {
		totalElements = #allGui,
		totalIssues = #issues,
		issues = issues,
	}, nil
end

return UIInspector
