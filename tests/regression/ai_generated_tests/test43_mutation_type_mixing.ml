// Test 43: Trying to mix mutable containers with different types
let create_mutable_container = fun (type t) (initial: t) -> (
    let cell = {mut value = initial};
    let setter = fun new_value -> cell.value <- new_value;
    let getter = fun _ -> cell.value;
    {cell; set = setter; get = getter}
);

let int_container = create_mutable_container 42;
let str_container = create_mutable_container "hello";

// Can we assign one container to another?
let mixed = if true then int_container else str_container;

// What happens if we try to set an int on the mixed container?
mixed.set 123;