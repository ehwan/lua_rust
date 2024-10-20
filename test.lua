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

print(math.pi)

print( math.abs( -math.pi ) )
print( math.abs( "3.5" ) )
print( math.abs( "-13.2" ) )

math.randomseed( )
print( math.random(10) )
print( math.random() )
print( math.random( 5, 10 ) )

print( math.deg(math.atan( 1 )) )

print( math.modf( 3.14 ) )
print( math.modf( 5 ) )

a = { 1, 2, 3, 4, 5, 6 }
print( a )
print( a[1], a[2], a[3] )
print( #a )

print( table.concat( a, ", ", 3, 5 ) )

table.insert( a, 3, 100 )
print( table.concat( a, ", ", 1, 7 ) )

a = table.pack( 'a', 'b', 'c', 'd', 'e' )
print( table.concat(a, ", ") )
a[10] = "ten"
print( table.unpack( a, 7, 9 ) )

print( "removed: "..tostring(table.remove( a )) )
print( table.unpack( a ) )
print( "removed: "..tostring(table.remove( a, 2 )) )
print( table.unpack( a ) )


a = { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 }
b = { 'a', 'b', 'c' }
table.move( a, 5, 9, 3, b )
print( table.unpack( a ) )
print( table.unpack( b ) )

print( 'generic-for loop with pairs' )
for k, v in pairs(a) do
    print( k, v )
end

print( 'generic-for loop with ipairs' )
for k, v in ipairs(a) do
    print( k, v )
end

print( 'sorting' )
a = { 3, 5, 1, 4, 2 }
table.sort( a )
print( table.unpack( a ) )

print( 'coroutine.running()' )
print( coroutine.running() )

print( 'coroutine.status()' )
print( coroutine.status( coroutine.running() ) )



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