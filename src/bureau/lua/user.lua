-- Add lua and plugin directories to loader path.
package.path = "plugins/?.lua;" .. package.path

local ftbl = ...

local set_pos = ftbl.set_pos
local set_rot = ftbl.set_rot
local send_msg = ftbl.send_msg
local send_packet = ftbl.send_packet
local disconnect = ftbl.disconnect

---@class User
---@field id number
---@field name string
---@field avatar string
---@field ip string
---@field _pos Vector
---@field _rot Basis
local user_meta = {}
user_meta.__index = user_meta

--- Disconnect the user from the bureau.
function user_meta:disconnect()
	return disconnect(self.id)
end

--- Set User's position.
---@param pos Vector
function user_meta:setPos(pos)
	self.pos = pos:clone()
	return set_pos(self.id, pos[1], pos[2], pos[3])
end

--- Get User's current position.
---@return Vector
function user_meta:getPos()
	return self._pos:clone()
end

--- Set User's rotation.
---@param rot Basis
function user_meta:setRot(rot)
	return set_rot(self.id, rot)
end

--- Set User's rotation.
---@return Basis
function user_meta:getRot()
	return self._rot:clone()
end

--- Send a packet to the User.
---@param msg string
function user_meta:sendMsg(msg)
	send_msg(self.id, msg)
end

--- Send a packet to the User.
---@param msg string
function user_meta:sendPacket(msg)
	send_packet(self.id, msg)
end

function user_meta:__tostring()
	return string.format("User: '%s' (%s)", self.name, self.id)
end

local users = {}

---@diagnostic disable-next-line: lowercase-global
user_manager = {}

--- Get all connected users.
---@return User[]
function user_manager.getAll()
	local ret = {}
	for _, user in pairs(users) do
		table.insert(ret, user)
	end
	return ret
end

--- Get user by their id.
---@param id number
---@return User
function user_manager.get(id)
	return users[id]
end

return users, user_meta

