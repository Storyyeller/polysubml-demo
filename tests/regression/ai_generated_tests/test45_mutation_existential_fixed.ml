// Test 45: Mutation with existential types (fixed)
let container1 = {mut value = 42; transform = fun x -> x + 1};
let container2 = {mut value = "hello"; transform = fun x -> x ^ " world"};

let mixed = if true then container1 else container2;

// Destructure without mut - can we still mutate?
let {type t; value: t; transform: t -> t} = mixed;

// This should fail - value is not mutable in the pattern
print value;