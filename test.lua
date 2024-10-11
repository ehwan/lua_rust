a = _VERSION
print(a)
print( select( '#', 1, 2, 3,4 , 5) )
print( select( 2, 1, 2, 3, 4, 5) )
print( select( -2, 1,2, 3, 4, 5 ) )
print( select( 10, 1,2, 3, 4, 5) )
print( string.byte(a, 1, 5) )
print( string.rep( a, 3, "::" ) )
lowered = string.lower(a)
uppered = string.upper(lowered)
print( lowered )
print( uppered )



function func()
    return 1, 2
end

local a, b, c, d = 4,5, 6, 7

print( a, b, c, d )

a, b, c, d = func()


print( a, b, c, d )

function factorial( n )
    if n == 0 then
        return 1;
    else
        return n * factorial(n - 1);
    end
end

for i = 1, 10, 2 do
    print( "Factorial of "..i.." is: ".. factorial(i) );
end


do
    return 0;
end

print(10);