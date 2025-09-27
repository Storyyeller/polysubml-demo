// Test 25: Function application associativity
let f = fun x -> x + 1;
let g = fun x -> x * 2;
let h = fun x -> x - 3;

// Test that a b c is parsed as a (b c), not (a b) c
let apply_to = fun x -> fun f -> f x;
print apply_to 5 f; // Should be apply_to 5 (f), which is (fun f -> f 5) f = f 5 = 6

// Test more complex case
let const = fun x -> fun y -> x;
print const 10 20; // Should be const 10 (20) = (fun y -> 10) 20 = 10