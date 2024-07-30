local function fact(n)
    if (n == 0) then
        return 1;
    else
        return n * fact(n - 1);
    end
end

local x = "test"
print("factorial value:");
"print(fact(5));
