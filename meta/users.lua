---@meta

----- Add plugin directories to loader path.
package.path = "plugins/?.lua;" .. package.path

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
function user_meta:disconnect() end

--- Set User's position.
---@param pos Vector
function user_meta:set_pos(pos) end

--- Get User's current position.
---@return Vector
function user_meta:pos() end

--- Set User's rotation.
---@param rot Basis
function user_meta:set_rot(rot) end

--- Get User's rotation.
---@return Basis
function user_meta:rot() end

--- Send a message to the User's chat.
---@param msg string
function user_meta:send_msg(msg) end

--- Send a packet to the User.
---@param msg string
function user_meta:send_packet(msg) end

function user_meta:__tostring() end

---@class userslib
local user_manager = {}

--- Get all connected users.
---@return User[]
function user_manager.all() end

--- Get user by their id.
---@param id number
---@return User
function user_manager.get(id) end

return user_manager
