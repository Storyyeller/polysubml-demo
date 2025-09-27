// Test 42: Mutation with polymorphic functions (fixed)
let create_mutable_container = fun (type t) (initial: t) -> (
    let cell = {mut value = initial};
    let setter = fun new_value -> cell.value <- new_value;
    let getter = fun _ -> cell.value;
    {cell; set = setter; get = getter}
);

let int_container = create_mutable_container 42;
let str_container = create_mutable_container "hello";

print int_container.cell.value;
print str_container.cell.value;

// Mutate them
int_container.set 99;
str_container.set "world";

print int_container.cell.value;
print str_container.cell.value;