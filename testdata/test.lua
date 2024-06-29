---@meta

global_int = 123

global_string = "hello"

global_table = {}

global_table.set_field = 123

---Setting an index is useful I guess?
global_table[1] = 456;

---What about this?
global_table[2] = function(x) end

---@param x integer
---@return integer
function global_table:set_method(x) end

---@class AClass
---@field foo string
local AClass = {}

---Does stuff
function AClass:do_stuff() end