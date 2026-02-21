--!strict
-- AutoDocs: Auto-generate Markdown documentation for ModuleScripts

local TreeWalker = require(script.Parent.Parent.Utils.TreeWalker)

return function(args: { [string]: any }): (boolean, any, string?)
	local targetPath = args.path or ""
	local docs: { string } = { "# StudioLink Auto-Generated Documentation\n" }
	local scripts = TreeWalker.collectScripts()

	for _, scriptInstance in ipairs(scripts) do
		if not scriptInstance:IsA("ModuleScript") then continue end
		if targetPath ~= "" and not scriptInstance:GetFullName():find(targetPath) then continue end

		local ok, source = pcall(function() return (scriptInstance :: any).Source end)
		if not ok or not source or source == "" then continue end

		local fullName = scriptInstance:GetFullName()
		table.insert(docs, "## " .. scriptInstance.Name)
		table.insert(docs, "**Path:** `" .. fullName .. "`\n")

		-- Extract module description from top comment
		local topComment = source:match("^%-%-([^\n]+)")
		if topComment then
			table.insert(docs, topComment:gsub("^%s*", "") .. "\n")
		end

		-- Find all function definitions
		local functions: { any } = {}

		-- Pattern: function ModuleName.functionName(args)
		for funcName, funcArgs in source:gmatch("function%s+%w+%.(%w+)%s*%((.-)%)") do
			table.insert(functions, { name = funcName, args = funcArgs })
		end

		-- Pattern: function ModuleName:methodName(args)
		for funcName, funcArgs in source:gmatch("function%s+%w+:(%w+)%s*%((.-)%)") do
			table.insert(functions, { name = funcName, args = funcArgs, isMethod = true })
		end

		-- Pattern: ModuleName.functionName = function(args)
		for funcName, funcArgs in source:gmatch("(%w+)%s*=%s*function%s*%((.-)%)") do
			table.insert(functions, { name = funcName, args = funcArgs })
		end

		if #functions > 0 then
			table.insert(docs, "### Functions\n")
			for _, func in ipairs(functions) do
				local prefix = if func.isMethod then ":" else "."
				table.insert(docs, "#### `" .. prefix .. func.name .. "(" .. func.args .. ")`")

				-- Try to find a comment above the function
				local pattern = "%-%-([^\n]+)\n[^\n]*function[^\n]*" .. func.name
				local comment = source:match(pattern)
				if comment then
					table.insert(docs, comment:gsub("^%s*", ""))
				end

				-- Parse arguments
				if func.args ~= "" then
					table.insert(docs, "\n**Parameters:**")
					for arg in func.args:gmatch("([^,]+)") do
						arg = arg:gsub("^%s+", ""):gsub("%s+$", "")
						if arg ~= "self" then
							-- Check for type annotation
							local name, typeAnnotation = arg:match("(%w+)%s*:%s*(.+)")
							if name and typeAnnotation then
								table.insert(docs, "- `" .. name .. "`: " .. typeAnnotation)
							else
								table.insert(docs, "- `" .. arg .. "`")
							end
						end
					end
				end
				table.insert(docs, "")
			end
		end

		-- Find require() dependencies
		local deps: { string } = {}
		for req in source:gmatch("require%((.-)%)") do
			local depName = req:match("%.(%w+)%s*$") or req
			table.insert(deps, depName)
		end

		if #deps > 0 then
			table.insert(docs, "### Dependencies")
			for _, dep in ipairs(deps) do
				table.insert(docs, "- " .. dep)
			end
			table.insert(docs, "")
		end

		table.insert(docs, "---\n")
	end

	local markdown = table.concat(docs, "\n")
	return true, markdown, nil
end
