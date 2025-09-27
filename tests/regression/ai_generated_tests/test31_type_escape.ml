// Test 31: Type variable escape via inference
// Try to create a situation where a type variable might escape its scope
let create_container = fun (type t) (x: t) -> {
    value = x;
    // Try to return a function that references the type variable
    get = fun () -> x;
    // The container should have existential type, not expose t
};