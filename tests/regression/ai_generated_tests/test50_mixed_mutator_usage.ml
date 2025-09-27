// Test 50: Using mixed mutator with different types
let create_escaping_ref = fun (type t) (x: t) -> (
    let cell = {mut value = x};
    fun new_value -> cell.value <- new_value
);

let int_mutator = create_escaping_ref 42;
let str_mutator = create_escaping_ref "hello";

let mixed_mutator = if true then int_mutator else str_mutator;

// What happens if we try to call it with an int?
print mixed_mutator 123;

// What about with a string?
print mixed_mutator "test";