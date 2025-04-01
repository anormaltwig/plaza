local users, user_meta = ...

local hook = {}

local function ins_tbl_ret(tbl, obj)
	local pos = #tbl + 1
	tbl[pos] = obj
	return obj
end

local think_hooks = {}
function hook.think(fn)
	return ins_tbl_ret(think_hooks, fn)
end

local user_connect_hooks = {}
function hook.user_connect(fn)
	return ins_tbl_ret(user_connect_hooks, fn)
end

local new_user_hooks = {}
function hook.new_user(fn)
	return ins_tbl_ret(new_user_hooks, fn)
end

local pos_update_hooks = {}
function hook.position_update(fn)
	return ins_tbl_ret(pos_update_hooks, fn)
end

local trans_update_hooks = {}
function hook.transform_update(fn)
	return ins_tbl_ret(trans_update_hooks, fn)
end

local chat_send_hooks = {}
function hook.chat_send(fn)
	return ins_tbl_ret(chat_send_hooks, fn)
end

local name_change_hooks = {}
function hook.name_change(fn)
	return ins_tbl_ret(name_change_hooks, fn)
end

local avatar_change_hooks = {}
function hook.avatar_change(fn)
	return ins_tbl_ret(avatar_change_hooks, fn)
end

local private_chat_hooks = {}
function hook.private_chat(fn)
	return ins_tbl_ret(private_chat_hooks, fn)
end

local aura_enter_hooks = {}
function hook.aura_enter(fn)
	return ins_tbl_ret(aura_enter_hooks, fn)
end

local aura_leave_hooks = {}
function hook.aura_leave(fn)
	return ins_tbl_ret(aura_leave_hooks, fn)
end

local user_disconnect_hooks = {}
function hook.user_disconnect(fn)
	return ins_tbl_ret(user_disconnect_hooks, fn)
end

local plugins_loaded_hooks = {}
function hook.plugins_loaded(fn)
	return ins_tbl_ret(plugins_loaded_hooks, fn)
end

package.loaded["hook"] = hook

local function run_hooks(tbl, ...)
	for i = 1, #tbl do
		local fn = tbl[i]
		local ret = fn(...)

		if ret then
			return ret
		end
	end
end

local Vector = require("vector")
local Basis = require("basis")

return {
	think = function()
		return run_hooks(think_hooks)
	end,
	user_connect = function(addr)
		return run_hooks(user_connect_hooks, addr)
	end,
	new_user = function(id, name, avatar, ip)
		local u = setmetatable({
			id = id,
			name = name,
			avatar = avatar,
			ip = ip,
			_pos = Vector(0, 0, 0),
			_rot = Basis(),
		}, user_meta)
		users[id] = u

		return run_hooks(new_user_hooks, u, name, avatar)
	end,
	pos_update = function(id, x, y, z)
		local user = users[id]
		user._pos = Vector(x, y, z)

		return run_hooks(pos_update_hooks, users[id], Vector(x, y, z))
	end,
	trans_update = function(id, arr)
		local user = users[id]
		local rot = Basis()
		rot:set(arr)
		user._rot = rot

		return run_hooks(trans_update_hooks, users[id])
	end,
	chat_send = function(id, msg)
		return run_hooks(chat_send_hooks, users[id], msg)
	end,
	name_change = function(id, name)
		local u = users[id]
		if not u then return end

		local old = u.name
		u.name = name

		return run_hooks(name_change_hooks, u, name, old)
	end,
	avatar_change = function(id, avatar)
		local u = users[id]
		if not u then return end

		local old = u.avatar
		u.avatar = avatar

		return run_hooks(avatar_change_hooks, u, avatar, old)
	end,
	private_chat = function(id1, id2, msg)
		return run_hooks(private_chat_hooks, users[id1], users[id2], msg)
	end,
	aura_enter = function(id1, id2)
		local u1 = users[id1]
		local u2 = users[id2]

		return run_hooks(aura_enter_hooks, u1, u2)
	end,
	aura_leave = function(id1, id2)
		local u1 = users[id1]
		local u2 = users[id2]

		return run_hooks(aura_leave_hooks, u1, u2)
	end,
	user_disconnect = function(id)
		local u = users[id]
		users[id] = nil

		return run_hooks(user_disconnect_hooks, u)
	end,
	plugins_loaded = function()
		return run_hooks(plugins_loaded_hooks)
	end
}
