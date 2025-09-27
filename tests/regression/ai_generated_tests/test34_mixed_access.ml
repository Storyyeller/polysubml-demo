// Test 34: Accessing mixed container values
let create_container = fun (type t) (x: t) -> (
    let getter = fun _ -> x;
    {value = x; get = getter}
);

let container1 = create_container 42;
let container2 = create_container "hello";

let mixed = if true then container1 else container2;

// What type is mixed.value?
print mixed.value;

// What about if we try arithmetic on it?
print mixed.value + 1;