// Test 40: Higher-rank types with explicit instantiation
let apply_id_to_both = fun (f: type t. t -> t) -> fun pair -> (
    let (a, b) = pair;
    (f a, f b)
);

let id = fun (type t) (x: t): t -> x;

// Try explicit instantiation as suggested
let result1 = apply_id_to_both id[] (42, "hello");
print result1;