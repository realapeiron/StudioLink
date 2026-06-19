--!strict
-- Pure heuristics for the security_scan RemoteEvent/RemoteFunction validation
-- check. Extracted from SecurityAudit so it can be unit-tested under Lune.
--
-- A security scanner must avoid FALSE NEGATIVES (calling an unvalidated remote
-- "validated"): those hide real vulnerabilities. Two old bugs caused exactly
-- that and are fixed here:
--   1. Remote names were fed to string.find as Lua PATTERNS, so a name like
--      "Data.Save" matched "DataXSave" (`.` = any char) — falsely linking a
--      remote to an unrelated script. Now matched as plain text.
--   2. A bare `if ... then` anywhere in the script counted as "validation".
--      Almost every script has an `if`, so almost every remote looked
--      validated. Now only real type/assert guards count.

local RemoteValidation = {}

-- Does `source` literally reference `remoteName`? Plain-text search: the name
-- is data, not a pattern.
function RemoteValidation.mentions(source: string, remoteName: string): boolean
	return source:find(remoteName, 1, true) ~= nil
end

-- Does `source` contain a recognized argument-validation guard?
function RemoteValidation.hasValidation(source: string): boolean
	return source:find("typeof%(") ~= nil
		or source:find("type%(") ~= nil
		or source:find("assert%(") ~= nil
end

-- True only when a script both references the remote AND has a validation guard.
function RemoteValidation.validatesRemote(serverScriptSources: { string }, remoteName: string): boolean
	for _, source in ipairs(serverScriptSources) do
		if RemoteValidation.mentions(source, remoteName) and RemoteValidation.hasValidation(source) then
			return true
		end
	end
	return false
end

return RemoteValidation
