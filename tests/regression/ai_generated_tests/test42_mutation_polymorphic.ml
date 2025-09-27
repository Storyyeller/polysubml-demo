// Test 42: Mutation with polymorphic functions
// Can we store different types in the same mutable field using polymorphism?
let create_mutable_container = fun (type t) (initial: t) -> {
    mut value = initial;
    set = fun new_value -> value <- new_value;
    get = fun _ -> value;
};

let int_container = create_mutable_container 42;
let str_container = create_mutable_container "hello";

print int_container.value;
print str_container.value;

// Mutate them
int_container.set 99;
str_container.set "world";

print int_container.value;
print str_container.value;