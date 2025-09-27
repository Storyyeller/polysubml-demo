// Test 36: Interaction between polymorphic and existential types
// Can we pass an existential type to a polymorphic function?
let poly_id = fun (type t) (x: t): t -> x;

let existential_record = {type u; value=42; transform=fun (x: u) -> x};

// Destructure the existential
let {type u; value: u; transform: u -> u} = existential_record;

// Try to pass the existential value to the polymorphic function
let result = poly_id value;