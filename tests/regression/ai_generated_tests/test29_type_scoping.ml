// Test 29: Type variable scoping in nested contexts
// Test that type variables don't leak from inner to outer scope
let outer = fun (type t) (x: t) -> (
    let inner = fun (type t) (y: t) -> y;  // Shadows outer t
    inner x  // Should this work? x has outer t, inner expects inner t
);