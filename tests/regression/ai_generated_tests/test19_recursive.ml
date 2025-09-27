// Test 19: Recursive functions
let rec factorial = fun n ->
    if n <= 1 then
        1
    else
        n * factorial (n - 1);

print factorial 5;
print factorial 0;
print factorial 1;