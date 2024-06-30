---@meta bit

---Integer, Bit Operations, provided by <http://bitop.luajit.org>
---
---[Documentation](http://bitop.luajit.org/api.html)
---@class bitlib
bit = {}

---Normalizes a number to the numeric range for bit operations and returns it.
---This function is usually not needed since all bit operations already normalize
---all of their input arguments. Check the operational semantics for details.
---@param x integer
---@return integer
---@nodiscard
function bit.tobit(x) end

---Converts its first argument to a hex string. The number of hex digits is
---given by the absolute value of the optional second argument. Positive numbers
---between 1 and 8 generate lowercase hex digits. Negative numbers generate
---uppercase hex digits. Only the least-significant 4*|n| bits are used. The
---default is to generate 8 lowercase hex digits.
---@param x integer
---@param n integer
---@return string
---@nodiscard
function bit.tohex(x, n) end
---@param x integer
---@return string
---@nodiscard
function bit.tohex(x) end

---Returns the bitwise **not** of its argument.
---@param x integer
---@return integer
---@nodiscard
function bit.bnot(x) end

return bit