// Test 44: Accessing mixed mutable container
let create_mutable_container = fun (type t) (initial: t) -> (
    let cell = {mut value = initial};
    let setter = fun new_value -> cell.value <- new_value;
    let getter = fun _ -> cell.value;
    {cell; set = setter; get = getter}
);

let int_container = create_mutable_container 42;
let str_container = create_mutable_container "hello";

let mixed = if true then int_container else str_container;
mixed.set 123;

// What type is the value we get back?
let value = mixed.get {};
print value;

// Can we do arithmetic on it?
print value + 1;