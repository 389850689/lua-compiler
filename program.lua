-- factorial ahh program in lua.
local function fact(n)
	if n == 0 then
		return 1
	else
		return n * fact(n - 1)
	end
end

local n = 5
print("factorial value:")
print(fact(n))
