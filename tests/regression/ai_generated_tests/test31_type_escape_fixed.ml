// Test 31: Type variable escape via inference (fixed)
let create_container = fun (type t) (x: t) -> {
    value = x;
    // Return a function that references the type variable
    get = fun _ -> x;
};