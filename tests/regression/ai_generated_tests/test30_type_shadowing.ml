// Test 30: Type variable shadowing should create distinct types
let test = fun (type t) (x: t) -> (
    let inner = fun (type t) (y: t): t -> x;  // Return x (outer t) but promise inner t
    inner
);