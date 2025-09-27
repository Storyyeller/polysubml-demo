// Test 6: Nested function type inference (fixed)
let outer = fun x -> (
    let inner = fun y -> x + y;
    inner
);

let add5 = outer 5;
print add5 10;

// Test higher order function
let apply = fun (f, x) -> f x;
let double = fun x -> x * 2;
print apply (double, 7);