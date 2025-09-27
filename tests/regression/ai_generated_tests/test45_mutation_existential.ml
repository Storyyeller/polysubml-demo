// Test 45: Mutation with existential types
// Create a record with existential type and mutable field
let container1 = {mut value = 42; transform = fun x -> x + 1};
let container2 = {mut value = "hello"; transform = fun x -> x ^ " world"};

// Mix them using existential types
let mixed = if true then container1 else container2;

// Destructure with existential pattern
let {type t; mut value: t; transform: t -> t} = mixed;

// Use the transform function
let old_value = value <- transform value;
print old_value;
print value;