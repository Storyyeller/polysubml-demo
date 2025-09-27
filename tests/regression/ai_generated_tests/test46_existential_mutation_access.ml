// Test 46: Mutation access with existential types
let container1 = {mut value = 42; transform = fun x -> x + 1};
let container2 = {mut value = "hello"; transform = fun x -> x ^ " world"};

let mixed = if true then container1 else container2;

// Can we mutate the original mixed record directly?
let old = mixed.value <- 999;
print old;
print mixed.value;

// Now destructure and use transform
let {type t; value: t; transform: t -> t} = mixed;
let transformed = transform value;
print transformed;