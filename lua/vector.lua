---@class Vector
---@field x number
---@field y number
---@field z number
local vec = {}
vec.__index = vec

function vec:__add(other)
	return Vector(self[1] + other[1], self[2] + other[2], self[3] + other[3])
end

function vec:__sub(other)
	return Vector(self[1] - other[1], self[2] - other[2], self[3] - other[3])
end

---@param x number
---@param y number
---@param z number
---@return Vector
function Vector(x, y, z)
	return setmetatable({x or 0, y or 0, z or 0}, vec)
end

