local ftbl = ...

local set_pos = ftbl.set_pos
local get_pos = ftbl.get_pos
local set_rot = ftbl.set_rot
local get_rot = ftbl.get_rot
local send_msg = ftbl.send_msg
local send_packet = ftbl.send_packet
local disconnect = ftbl.disconnect
local get_peer_addr = ftbl.get_peer_addr

-- Add lua and plugin directories to loader path.
package.path = "lua/?.lua;plugins/?.lua;" .. package.path

require("basis")
require("hook")
require("vector")

---@class User
---@field id number
---@field name string
---@field avatar string
local user_meta = {}
user_meta.__index = user_meta

--- Disconnect the user from the bureau.
function user_meta:disconnect()
	return disconnect(self.id)
end

--- Get socket address of the user.
---@return string
function user_meta:getPeerAddr()
	return get_peer_addr(self.id)
end

--- Set User's position.
---@param pos Vector
function user_meta:setPos(pos)
	return set_pos(self.id, pos[1], pos[2], pos[3])
end

--- Get User's current position.
---@return Vector
function user_meta:getPos()
	return Vector(get_pos(self.id))
end

--- Set User's rotation.
---@param rot Basis
function user_meta:setRot(rot)
	return set_rot(self.id, rot)
end

--- Set User's rotation.
---@return Basis
function user_meta:getRot()
	local rot = Basis()
	rot:set(get_rot(self.id))
	return rot
end

--- Send a message to the User's chat.
---@param msg string
function user_meta:sendMsg(msg)
	send_msg(self.id, msg)
end

--- Send a message to the User's chat.
---@param msg string
function user_meta:sendPacket(msg)
	send_packet(self.id, msg)
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

return {
	think = function()
		return hook.call("Think")
	end,
	user_connecting = function(addr)
		return hook.call("UserConnecting", addr)
	end,
	new_user = function(id, name, avatar)
		local u = setmetatable({id = id, name = name, avatar = avatar}, user_meta)
		users[id] = u

		return hook.call("NewUser", u, name, avatar)
	end,
	pos_update = function(id, x, y, z)
		return hook.call("PositionUpdate", users[id], Vector(x, y, z))
	end,
	trans_update = function(id)
		return hook.call("TransformUpdate", users[id])
	end,
	chat_send = function(id, msg)
		return hook.call("ChatSend", users[id], msg)
	end,
	name_change = function(id, name)
		local u = users[id]
		if not u then return end

		local old = u.name
		u.name = name

		return hook.call("NameChange", u, name, old)
	end,
	avatar_change = function(id, avatar)
		local u = users[id]
		if not u then return end

		local old = u.avatar
		u.avatar = avatar

		return hook.call("AvatarChange", u, avatar, old)
	end,
	private_chat = function(id1, id2, msg)
		return hook.call("PrivateChat", users[id1], users[id2], msg)
	end,
	aura_enter = function(id1, id2)
		local u1 = users[id1]
		local u2 = users[id2]

		return hook.call("AuraEnter", u1, u2)
	end,
	aura_leave = function(id1, id2)
		local u1 = users[id1]
		local u2 = users[id2]

		return hook.call("AuraLeave", u1, u2)
	end,
	user_disconnect = function(id)
		local u = users[id]
		users[id] = nil

		return hook.call("UserDisconnect", u)
	end
}

