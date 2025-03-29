local users, user_meta = ...

---@diagnostic disable-next-line: lowercase-global
hook = {}

local function ins_tbl_ret(tbl, obj)
	local pos = #tbl + 1
	tbl[pos] = obj
	return obj
end

local think_hooks = {}
---@param fn fun()
---@return integer
function hook.onThink(fn)
	return ins_tbl_ret(think_hooks, fn)
end

local user_connect_hooks = {}
---@param fn fun(addr: string):boolean?
---@return integer
function hook.onUserConnect(fn)
	return ins_tbl_ret(user_connect_hooks, fn)
end

local new_user_hooks = {}
---@param fn fun(user: User, name: string, avatar: string)
---@return integer
function hook.onNewUser(fn)
	return ins_tbl_ret(new_user_hooks, fn)
end

local pos_update_hooks = {}
---@param fn fun(user: User, pos: Vector)
---@return integer
function hook.onPositionUpdate(fn)
	return ins_tbl_ret(pos_update_hooks, fn)
end

local trans_update_hooks = {}
---@param fn fun(user: User)
---@return integer
function hook.onTransformUpdate(fn)
	return ins_tbl_ret(trans_update_hooks, fn)
end

local chat_send_hooks = {}
---@param fn fun(user: User, msg: string):string?
---@return integer
function hook.onChatSend(fn)
	return ins_tbl_ret(chat_send_hooks, fn)
end

local name_change_hooks = {}
---@param fn fun(user: User, name: string, old: string)
---@return integer
function hook.onNameChange(fn)
	return ins_tbl_ret(name_change_hooks, fn)
end

local avatar_change_hooks = {}
---@param fn fun(user: User, avatar: string, old: string)
---@return integer
function hook.onAvatarChange(fn)
	return ins_tbl_ret(avatar_change_hooks, fn)
end

local private_chat_hooks = {}
---@param fn fun(sender: User, receiver: User, msg: string):string?
---@return integer
function hook.onPrivateChat(fn)
	return ins_tbl_ret(private_chat_hooks, fn)
end

local aura_enter_hooks = {}
---@param fn fun(u1: User, u2: User)
---@return integer
function hook.onAuraEnter(fn)
	return ins_tbl_ret(aura_enter_hooks, fn)
end

local aura_leave_hooks = {}
---@param fn fun(u1: User, u2: User)
---@return integer
function hook.onAuraLeave(fn)
	return ins_tbl_ret(aura_leave_hooks, fn)
end

local user_disconnect_hooks = {}
---@param fn fun(user: User)
---@return integer
function hook.onUserDisconnect(fn)
	return ins_tbl_ret(user_disconnect_hooks, fn)
end

local plugins_loaded_hooks = {}
---@param fn fun()
---@return integer
function hook.onPluginsLoaded(fn)
	return ins_tbl_ret(plugins_loaded_hooks, fn)
end

local function run_hooks(tbl, ...)
	for i = 1, #tbl do
		local fn = tbl[i]
		local ret = fn(...)

		if ret then
			return ret
		end
	end
end

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

