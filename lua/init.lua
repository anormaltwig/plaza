local ftbl = ...

local set_pos = ftbl.set_pos
local get_pos = ftbl.get_pos
local set_rot = ftbl.set_rot
local get_rot = ftbl.get_rot
local send_msg = ftbl.send_msg

-- Add lua and plugin directories to loader path.
package.path = "lua/?.lua;plugins/?.lua;" .. package.path

---@class User
---@field id number
local user = {}
user.__index = user

--- Set User's position.
---@param pos Vector
function user:setPos(pos)
	return set_pos(self.id, pos[1], pos[2], pos[3])
end

--- Get User's current position.
---@return Vector
function user:getPos()
	return Vector(get_pos(self.id))
end

--- Set User's rotation.
---@param rot Basis
function user:setRot(rot)
	return set_rot(self.id, rot)
end

--- Set User's rotation.
---@return Basis
function user:getRot()
	local basis = Basis()
	basis:set(get_rot(self.id))
	return basis
end

--- Send a message to the User's chat.
---@param msg string
function user:sendMsg(msg)
	send_msg(self.id, msg)
end

--- Create new user from an id.
---@param id number
---@return User
function User(id)
	return setmetatable({id = id}, user)
end

---@class Vector
---@field x number
---@field y number
---@field z number
local vec = {}
vec.__index = vec

function vec:__add(other)
	return Vector(self[1] + other[1], self[2] + other[2], self[3] + other[3])
end

---@param x number
---@param y number
---@param z number
---@return Vector
function Vector(x, y, z)
	return setmetatable({x or 0, y or 0, z or 0}, vec)
end

---@class Basis
local basis = {}
basis.__index = basis

function basis:set(arr)
	for i = 1, 9 do
		self[i] = arr[i] or 0
	end
end

--- Sets values based on a given yaw. Only use this function on a new basis.
---@param yaw number yaw in radians
function basis:fromYaw(yaw)
	local s = math.sin(yaw)
	local c = math.cos(yaw)

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

--- Sets the scale of the basis
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

---@return Basis
function Basis()
	return setmetatable({
		1, 0, 0,
		0, 1, 0,
		0, 0, 1,
	}, basis)
end

require("hook")

return {
	think = function()
		return hook.call("Think")
	end,
	new_user = function(id, name, avatar)
		return hook.call("NewUser", User(id), name, avatar)
	end,
	pos_update = function(id, x, y, z)
		return hook.call("PositionUpdate", User(id), Vector(x, y, z))
	end,
	trans_update = function(id)
		return hook.call("TransformUpdate", User(id))
	end,
	chat_send = function(id, msg)
		return hook.call("ChatSend", User(id), msg)
	end,
	name_change = function(id, name)
		return hook.call("NameChange", User(id), name)
	end,
	avatar_change = function(id, avatar)
		return hook.call("AvatarChange", User(id), avatar)
	end,
}

