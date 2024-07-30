-- defines a factorial function
local function fact(n)
  if n == 0 then
    return 1
  else
    return n * fact(n - 1)
  end
end

print("factorial value:")
print(fact(5))
