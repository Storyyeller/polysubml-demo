// Test 33: Trying to mix containers with different escaped types
let create_container = fun (type t) (x: t) -> (
    let getter = fun _ -> x;
    {value = x; get = getter}
);

let container1 = create_container 42;
let container2 = create_container "hello";

// Try to assign one to the other - should this work?
let mixed = if true then container1 else container2;