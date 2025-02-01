local function test_refcell()
	local users = user_manager.getAll()
	for i, user in ipairs(users) do
		user:setPos(Vector(i * 2, 0, 0))
	end
end

for _, fn in pairs(hook) do
	fn(test_refcell)
end
