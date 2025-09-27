// Test 31: Type variable escape via inference (fixed2)
let create_container = fun (type t) (x: t) -> (
    let getter = fun _ -> x;
    {value = x; get = getter}
);