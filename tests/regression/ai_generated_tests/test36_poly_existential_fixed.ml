// Test 36: Interaction between polymorphic and existential types (fixed)
let poly_id = fun (type t) (x: t): t -> x;

// Create existential via inference
let existential_record = {value=42; transform=fun x -> x + 1};

// Destructure the existential with explicit type parameter
let {type u; value: u; transform: u -> u} = existential_record;

// Try to pass the existential value to the polymorphic function
let result = poly_id value;