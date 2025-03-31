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
---@field _pos Vector use user:pos() instead
---@field _rot Basis use user:rot() instead
local user_meta = {}
user_meta.__index = user_meta

--- Disconnect the user from the bureau.
function user_meta:disconnect()
	return disconnect(self.id)
end

--- Set User's position.
---@param pos Vector
function user_meta:set_pos(pos)
	self._pos = pos:clone()
	return set_pos(self.id, pos[1], pos[2], pos[3])
end

--- Get User's current position.
---@return Vector
function user_meta:pos()
	return self._pos:clone()
end

--- Set User's rotation.
---@param rot Basis
function user_meta:set_rot(rot)
	self._rot = rot
	return set_rot(self.id, rot)
end

--- Set User's rotation.
---@return Basis
function user_meta:rot()
	return self._rot:clone()
end

--- Send a message to the User's chat.
---@param msg string
function user_meta:send_msg(msg)
	send_msg(self.id, msg)
end

--- Send a packet to the User.
---@param msg string
function user_meta:send_packet(msg)
	send_packet(self.id, msg)
end

function user_meta:__tostring()
	return string.format("User: '%s' (%s)", self.name, self.id)
end

local users = {}

local user_manager = {}

--- Get all connected users.
---@return User[]
function user_manager.all()
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

package.loaded["users"] = user_manager

return users, user_meta
