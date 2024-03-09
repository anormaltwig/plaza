local hooks = {}

---@diagnostic disable-next-line: lowercase-global
hook = {}

--- Adds a new callback for the given hook name.
---@param name string
---@param id string
---@param func function
function hook.add(name, id, func)
	name = string.lower(name)
	local tbl = hooks[name]
	if not tbl then
		tbl = {}
		hooks[name] = tbl
	end

	tbl[id] = func
end

--- Remove callback for the given hook name and id pair.
---@param name string
---@param id string
function hook.remove(name, id)
	name = string.lower(name)
	hooks[name][id] = nil
end

--- Run a hook. Useful for letting other plugins know when something happens.
---@param name string
---@param ...any
function hook.call(name, ...)
	name = string.lower(name)

	local funcs = hooks[name]
	if not funcs then return end

	for _, func in pairs(funcs) do
		local ret = {func(...)}

		if #ret > 0 then
			return unpack(ret)
		end
	end
end

