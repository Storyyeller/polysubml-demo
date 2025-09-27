// Test 47: Compatible types with existential mutation
let container1 = {mut value = 42; transform = fun x -> x + 1};
let container2 = {mut value = 100; transform = fun x -> x * 2};

let mixed = if true then container1 else container2;

// Mutate and transform
let old = mixed.value <- 50;
print old;
print mixed.value;

// Destructure and transform
let {type t; value: t; transform: t -> t} = mixed;
let transformed = transform value;
print transformed;