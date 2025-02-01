---@class Basis
local basis = {}
basis.__index = basis

--- Set values of the basis.
---@param arr number[]
function basis:set(arr)
	for i = 1, 9 do
		self[i] = arr[i] or 0
	end
end

--- Sets values based on a given angle. Only use this function on a new basis.
---@param r number Yaw in radians.
function basis:fromYaw(r)
	local s = math.sin(r)
	local c = math.cos(r)

	self[1] = c
	self[3] = s
	self[7] = -s
	self[9] = c
end

--- Multiplies every value in the basis by n.
---@param n number
function basis:scale(n)
	for i = 1, 9 do
		self[i] = self[i] * n
	end
end

--- Gets scale of the basis.
---@return Vector
function basis:getScale()
	return Vector(
		math.sqrt(self[1]^2 + self[4]^2 + self[7]^2),
		math.sqrt(self[2]^2 + self[5]^2 + self[8]^2),
		math.sqrt(self[3]^2 + self[6]^2 + self[9]^2)
	)
end

--- Sets the scale of the basis.
---@param v Vector
function basis:setScale(v)
	local s = self:getScale()

	self[1] = self[1] / s[1] * v[1]
	self[2] = self[2] / s[1] * v[1]
	self[3] = self[3] / s[1] * v[1]

	self[4] = self[4] / s[2] * v[2]
	self[5] = self[5] / s[2] * v[2]
	self[6] = self[6] / s[2] * v[2]

	self[7] = self[7] / s[3] * v[3]
	self[8] = self[8] / s[3] * v[3]
	self[9] = self[9] / s[3] * v[3]
end

--- Clones the basis.
---@return Basis
function basis:clone()
	local rot = Basis()
	rot:set(self)
	return rot
end

--- Create a new basis.
---@return Basis
function Basis()
	return setmetatable({
		1, 0, 0,
		0, 1, 0,
		0, 0, 1,
	}, basis)
end

