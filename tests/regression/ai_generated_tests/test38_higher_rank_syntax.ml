// Test 38: Finding correct higher-rank type syntax
// Based on README, should be: type t. t -> t
let apply_id_to_both = fun (f: type t. t -> t) -> fun pair -> (
    let (a, b) = pair;
    (f a, f b)
);