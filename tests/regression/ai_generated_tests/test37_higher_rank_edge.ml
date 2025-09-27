// Test 37: Higher-rank type inference edge cases
// Test higher-rank function with existential types
let apply_to_both = fun (f: type t. t -> t) (pair: (int, str)) -> (
    let (a, b) = pair;
    (f a, f b)
);

// This should work - id function is polymorphic enough
let id = fun (type t) (x: t): t -> x;
let result1 = apply_to_both id (42, "hello");
print result1;

// This should fail - not polymorphic enough
let add_one = fun x -> x + 1;
let result2 = apply_to_both add_one (42, "hello");