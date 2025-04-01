-- Add plugin directories to loader path.
package.path = "plugins/?.lua;" .. package.path

local ftbl = ...

local set_pos = ftbl.set_pos
local set_rot = ftbl.set_rot
local send_msg = ftbl.send_msg
local send_packet = ftbl.send_packet
local disconnect = ftbl.disconnect

local user_meta = {}
user_meta.__index = user_meta

function user_meta:disconnect()
	return disconnect(self.id)
end

function user_meta:set_pos(pos)
	self._pos = pos:clone()
	return set_pos(self.id, pos[1], pos[2], pos[3])
end

function user_meta:pos()
	return self._pos:clone()
end

function user_meta:set_rot(rot)
	self._rot = rot
	return set_rot(self.id, rot)
end

function user_meta:rot()
	return self._rot:clone()
end

function user_meta:send_msg(msg)
	send_msg(self.id, msg)
end

function user_meta:send_packet(msg)
	send_packet(self.id, msg)
end

function user_meta:__tostring()
	return string.format("User: '%s' (%s)", self.name, self.id)
end

local users = {}

---@class userslib
local user_manager = {}

function user_manager.all()
	local ret = {}
	for _, user in pairs(users) do
		table.insert(ret, user)
	end
	return ret
end

function user_manager.get(id)
	return users[id]
end

package.loaded["users"] = user_manager

return users, user_meta
