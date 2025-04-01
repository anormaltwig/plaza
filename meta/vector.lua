---@meta

---@class Vector
---@field x number
---@field y number
---@field z number
local vec = {}
vec.__index = vec

--- Create a new vector, you can get x, y, and z components by indexing 1, 2, and 3 respectively.
---@param x number
---@param y number
---@param z number
---@return Vector
local function Vector(x, y, z) end

---@return string
function vec.__tostring(a) end

---@return Vector
function vec.__add(a, b) end

---@return Vector
function vec.__sub(a, b) end

---@return Vector
function vec.__mul(a, b) end

---@return Vector
function vec.__div(a, b) end

---@return Vector
function vec.__unm(a) end

---@return Vector
function vec.__eq(a, b) end

--- Get the length of the vector squared. (Faster than getting the actual length)
---@return number
function vec:length_sqr() end

--- Get the length of the vector.
---@return number
function vec:length() end

--- Create a new vector with the same direction but a length of 1.
---@return Vector
function vec:normalized() end

--- Modify the vector so that its length is 1.
function vec:normalize() end

--- Clone vector.
---@return Vector
function vec:clone() end

return Vector
