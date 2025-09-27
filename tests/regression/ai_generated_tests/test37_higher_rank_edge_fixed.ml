// Test 37: Higher-rank type inference edge cases (fixed)
// Test without explicit higher-rank annotation first
let apply_to_both = fun f -> fun pair -> (
    let (a, b) = pair;
    (f a, f b)
);

// This should work - id function is polymorphic enough
let id = fun (type t) (x: t): t -> x;
let result1 = apply_to_both id (42, "hello");
print result1;