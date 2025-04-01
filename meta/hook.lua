---@meta

---@class hooklib
local hook = {}

---@param fn fun()
---@return integer
function hook.think(fn) end

---@param fn fun(addr: string):boolean?
---@return integer
function hook.user_connect(fn) end

---@param fn fun(user: User, name: string, avatar: string)
---@return integer
function hook.new_user(fn) end

---@param fn fun(user: User, pos: Vector)
---@return integer
function hook.position_update(fn) end

---@param fn fun(user: User)
---@return integer
function hook.transform_update(fn) end

---@param fn fun(user: User, msg: string):string?
---@return integer
function hook.chat_send(fn) end

---@param fn fun(user: User, name: string, old: string)
---@return integer
function hook.name_change(fn) end

---@param fn fun(user: User, avatar: string, old: string)
---@return integer
function hook.avatar_change(fn) end

---@param fn fun(sender: User, receiver: User, msg: string):string?
---@return integer
function hook.private_chat(fn) end

---@param fn fun(u1: User, u2: User)
---@return integer
function hook.aura_enter(fn) end

---@param fn fun(u1: User, u2: User)
---@return integer
function hook.aura_leave(fn) end

---@param fn fun(user: User)
---@return integer
function hook.user_disconnect(fn) end

---@param fn fun()
---@return integer
function hook.plugins_loaded(fn) end

return hook
