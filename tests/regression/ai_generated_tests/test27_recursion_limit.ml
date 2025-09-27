// Test 27: Recursion limit test
let rec deep_recursion = fun n ->
    if n <= 0 then
        0
    else
        deep_recursion (n - 1);

// This should work fine
print deep_recursion 100;

// This might hit recursion limit based on README mentioning ~16000
print deep_recursion 20000;