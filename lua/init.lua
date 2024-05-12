local ftbl = ...

local set_pos = ftbl.set_pos
local set_rot = ftbl.set_rot
local send_msg = ftbl.send_msg
local send_packet = ftbl.send_packet
local disconnect = ftbl.disconnect

-- Add lua and plugin directories to loader path.
package.path = "lua/?.lua;plugins/?.lua;" .. package.path

require("hook")
require("vector")
require("basis")

---@class User
---@field id number
---@field name string
---@field avatar string
---@field _ip string
---@field _pos Vector
---@field _rot Basis
local user_meta = {}
user_meta.__index = user_meta

--- Disconnect the user from the bureau.
function user_meta:disconnect()
	return disconnect(self.id)
end

--- Get the IP address of the user.
---@return string
function user_meta:getIp()
	return self._ip
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
	user_connect = function(addr)
		return hook.call("UserConnecting", addr)
	end,
	new_user = function(id, name, avatar, ip)
		local u = setmetatable({
			id = id,
			name = name,
			avatar = avatar,
			_ip = ip,
			_pos = Vector(0, 0, 0),
			_rot = Basis(),
		}, user_meta)
		users[id] = u

		return hook.call("NewUser", u, name, avatar)
	end,
	pos_update = function(id, x, y, z)
		local user = users[id]
		user._pos = Vector(x, y, z)

		return hook.call("PositionUpdate", users[id], Vector(x, y, z))
	end,
	trans_update = function(id, arr)
		local user = users[id]
		local rot = Basis()
		rot:set(arr)
		user._rot = rot

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

