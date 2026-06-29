--!strict
-- Fast buffer→base64 encoder. viewport_capture ships multi-MB RGBA, so this is
-- buffer-based (no per-char string concat) and bit32-driven. Pure logic —
-- unit-tested under Lune against RFC 4648 vectors.

local Base64 = {}

local ALPHABET = buffer.fromstring("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/")
local PAD = 61 -- '='

function Base64.encode(input: buffer): string
	local len = buffer.len(input)
	if len == 0 then
		return ""
	end
	local outLen = math.floor((len + 2) / 3) * 4
	local out = buffer.create(outLen)
	local i, o = 0, 0
	while i < len do
		local b1 = buffer.readu8(input, i)
		local b2 = if i + 1 < len then buffer.readu8(input, i + 1) else 0
		local b3 = if i + 2 < len then buffer.readu8(input, i + 2) else 0
		local n = bit32.bor(bit32.lshift(b1, 16), bit32.lshift(b2, 8), b3)
		buffer.writeu8(out, o, buffer.readu8(ALPHABET, bit32.band(bit32.rshift(n, 18), 63)))
		buffer.writeu8(out, o + 1, buffer.readu8(ALPHABET, bit32.band(bit32.rshift(n, 12), 63)))
		buffer.writeu8(out, o + 2, if i + 1 < len then buffer.readu8(ALPHABET, bit32.band(bit32.rshift(n, 6), 63)) else PAD)
		buffer.writeu8(out, o + 3, if i + 2 < len then buffer.readu8(ALPHABET, bit32.band(n, 63)) else PAD)
		i += 3
		o += 4
	end
	return buffer.tostring(out)
end

-- Convenience for tests / small strings.
function Base64.encodeString(s: string): string
	return Base64.encode(buffer.fromstring(s))
end

return Base64
