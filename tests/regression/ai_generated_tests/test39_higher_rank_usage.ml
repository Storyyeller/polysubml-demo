// Test 39: Using higher-rank types
let apply_id_to_both = fun (f: type t. t -> t) -> fun pair -> (
    let (a, b) = pair;
    (f a, f b)
);

// This should work
let id = fun (type t) (x: t): t -> x;
let result1 = apply_id_to_both id (42, "hello");
print result1;

// This should fail
let add_one = fun x -> x + 1;
let result2 = apply_id_to_both add_one (42, "hello");