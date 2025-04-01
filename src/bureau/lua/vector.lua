local vec = {}
vec.__index = vec

--- Create a new vector, you can get x, y, and z components by indexing 1, 2, and 3 respectively.
local function Vector(x, y, z)
	return setmetatable({x or 0, y or 0, z or 0}, vec)
end

function vec.__tostring(a)
	return string.format("[x: %d, y: %d, z: %d]", a[1], a[2], a[3])
end

function vec.__add(a, b)
	return setmetatable({a[1] + b[1], a[2] + b[2], a[3] + b[3]}, vec)
end

function vec.__sub(a, b)
	return setmetatable({a[1] - b[1], a[2] - b[2], a[3] - b[3]}, vec)
end

function vec.__mul(a, b)
	if type(a) == "number" then
		return setmetatable({b[1] * a, b[2] * a, b[3] * a}, vec)
	elseif type(b) == "number" then
		return setmetatable({a[1] * b, a[2] * b, a[3] * b}, vec)
	elseif getmetatable(a) == getmetatable(b) then
		return setmetatable({a[1] * b[1], a[2] * b[2], a[3] * b[3]}, vec)
	end
end

function vec.__div(a, b)
	if type(a) == "number" then
		return setmetatable({b[1] / a, b[2] / a, b[3] / a}, vec)
	elseif type(b) == "number" then
		return setmetatable({a[1] / b, a[2] / b, a[3] / b}, vec)
	elseif getmetatable(a) == getmetatable(b) then
		return setmetatable({a[1] / b[1], a[2] / b[2], a[3] / b[3]}, vec)
	end
end

function vec.__unm(a)
	return setmetatable({-a[1], -a[2], -a[3]}, vec)
end

function vec.__eq(a, b)
	return a[1] == b[1] and a[2] == b[2] and a[3] == b[3]
end

function vec:length_sqr()
	return self[1]^2 + self[2]^2 + self[3]^2
end

function vec:length()
	return math.sqrt(self[1]^2 + self[2]^2 + self[3]^2)
end

function vec:normalized()
	local len = math.sqrt(self[1]^2 + self[2]^2 + self[3]^2)
	return Vector(self[1] / len, self[2] / len, self[3] / len)
end

function vec:normalize()
	local len = math.sqrt(self[1]^2 + self[2]^2 + self[3]^2)

	self[1] = self[1] / len
	self[2] = self[2] / len
	self[3] = self[3] / len
end

function vec:clone()
	return Vector(self[1], self[2], self[3])
end

package.loaded["vector"] = Vector
