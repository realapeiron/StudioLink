--!strict
-- AnimationInspect: Inspect animations in the place

local TreeWalker = require(script.Parent.Parent.Utils.TreeWalker)

local AnimationInspect = {}

function AnimationInspect.list(args: { [string]: any }): (boolean, any, string?)
	local animations: { any } = {}

	-- Find all Animation objects
	TreeWalker.walkDescendants(game:GetService("Workspace"), function(instance)
		if instance:IsA("Animation") then
			table.insert(animations, {
				name = instance.Name,
				path = instance:GetFullName(),
				animationId = (instance :: Animation).AnimationId,
			})
		end
	end)

	-- Also check for AnimationController and Animator
	local animators: { any } = {}
	TreeWalker.walkDescendants(game:GetService("Workspace"), function(instance)
		if instance:IsA("Animator") or instance:IsA("AnimationController") then
			table.insert(animators, {
				name = instance.Name,
				path = instance:GetFullName(),
				className = instance.ClassName,
			})
		end
	end)

	-- Check scripts for animation references
	local scriptRefs: { any } = {}
	local scripts = TreeWalker.collectScripts()
	for _, scriptInstance in ipairs(scripts) do
		local ok, source = pcall(function() return (scriptInstance :: any).Source end)
		if ok and source then
			for animId in source:gmatch("rbxassetid://(%d+)") do
				table.insert(scriptRefs, {
					animationId = "rbxassetid://" .. animId,
					referencedIn = scriptInstance:GetFullName(),
				})
			end
		end
	end

	return true, {
		animations = animations,
		animators = animators,
		scriptReferences = scriptRefs,
		totalAnimations = #animations,
		totalReferences = #scriptRefs,
	}, nil
end

function AnimationInspect.inspect(args: { [string]: any }): (boolean, any, string?)
	local animationId = args.animationId
	if not animationId then
		return false, nil, "animationId is required"
	end

	-- Find all references to this animation ID
	local references: { any } = {}
	local scripts = TreeWalker.collectScripts()

	for _, scriptInstance in ipairs(scripts) do
		local ok, source = pcall(function() return (scriptInstance :: any).Source end)
		if ok and source and source:find(animationId) then
			-- Find context around the animation usage
			for line in source:gmatch("[^\n]+") do
				if line:find(animationId) then
					table.insert(references, {
						script = scriptInstance:GetFullName(),
						line = line:gsub("^%s+", ""),
					})
				end
			end
		end
	end

	return true, {
		animationId = animationId,
		references = references,
		totalReferences = #references,
	}, nil
end

function AnimationInspect.conflicts(args: { [string]: any }): (boolean, any, string?)
	-- Analyze scripts for potential animation conflicts
	local animPlays: { { script: string, animId: string, priority: string? } } = {}
	local scripts = TreeWalker.collectScripts()

	for _, scriptInstance in ipairs(scripts) do
		local ok, source = pcall(function() return (scriptInstance :: any).Source end)
		if not ok or not source then continue end

		-- Find :Play() calls with animation context
		for line in source:gmatch("[^\n]+") do
			if line:find(":Play%(") and (line:find("Animation") or line:find("Track") or line:find("anim")) then
				local animId = source:match("rbxassetid://(%d+)")
				local priority = line:match("Priority%s*=%s*Enum%.AnimationPriority%.(%w+)")
				table.insert(animPlays, {
					script = scriptInstance:GetFullName(),
					animId = animId or "unknown",
					priority = priority,
				})
			end
		end
	end

	-- Detect potential conflicts (same script playing multiple animations)
	local conflicts: { any } = {}
	local scriptAnimCounts: { [string]: number } = {}

	for _, play in ipairs(animPlays) do
		scriptAnimCounts[play.script] = (scriptAnimCounts[play.script] or 0) + 1
	end

	for scriptPath, count in pairs(scriptAnimCounts) do
		if count > 1 then
			local anims = {}
			for _, play in ipairs(animPlays) do
				if play.script == scriptPath then
					table.insert(anims, play)
				end
			end
			table.insert(conflicts, {
				script = scriptPath,
				animationCount = count,
				animations = anims,
				warning = "Multiple animations played from same script — ensure proper priority and stopping",
			})
		end
	end

	return true, {
		totalAnimPlays = #animPlays,
		potentialConflicts = conflicts,
	}, nil
end

return AnimationInspect
