--!strict
local PlayHelpers = require(script.Parent.Parent.Utils.PlayHelpers)

return function(args: { [string]: any }): (boolean, any, string?)
	local ctxOk, ctxErr = PlayHelpers.requireContext("play")
	if not ctxOk then
		return false, nil, ctxErr
	end

	local text = args.text
	if type(text) ~= "string" then
		return false, nil, "text is required (string)"
	end

	local player, perr = PlayHelpers.resolvePlayer(args.player)
	if not player then
		return false, nil, perr
	end

	local instance, serr = PlayHelpers.resolveSelector(args.selector, player)
	if not instance then
		return false, nil, serr
	end

	if
		not (
			instance:IsA("TextBox")
			or instance:IsA("TextLabel")
			or instance:IsA("TextButton")
		)
	then
		return false,
			nil,
			"selector resolved to "
				.. instance.ClassName
				.. ", expected TextBox / TextLabel / TextButton"
	end

	local previousText = (instance :: any).Text
	;(instance :: any).Text = text

	return true,
		{
			target_path = instance:GetFullName(),
			class_name = instance.ClassName,
			previous_text = previousText,
			new_text = text,
		},
		nil
end
