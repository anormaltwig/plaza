---@meta

---@class Basis
local basis = {}
basis.__index = basis

--- Create a new basis.
---@return Basis
local function Basis() end

--- Set values of the basis.
---@param arr number[]
function basis:set(arr) end

--- Sets values based on a given angle. Only use this function on a new basis.
---@param r number Yaw in radians.
function basis:from_yaw(r) end

--- Gets scale of the basis.
---@return Vector
function basis:scale()end

--- Multiplies every value in the basis by n.
---@param n number
function basis:scale_by(n) end

--- Sets the scale of the basis.
---@param v Vector
function basis:set_scale(v) end

--- Clones the basis.
---@return Basis
function basis:clone() end

return Basis
