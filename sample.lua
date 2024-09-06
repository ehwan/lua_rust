-- Function to check if a number is even or odd
function checkEvenOrOdd(number)
    if number % 2 == 0 then
        return "even"
    else
        return "odd"
    end
end

-- Get user input
print("Enter a number: ")
userInput = io.read("*n")  -- Reads a number from user input

-- Check if the number is even or odd
result = checkEvenOrOdd(userInput)

-- Output the result
print("The number is " .. result)