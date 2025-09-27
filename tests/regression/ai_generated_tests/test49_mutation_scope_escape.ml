// Test 49: Mutation escaping type variable scopes
// Can we create a mutable reference that outlives its type variable scope?
let create_escaping_ref = fun (type t) (x: t) -> (
    let cell = {mut value = x};
    // Return a function that can mutate the cell
    fun new_value -> cell.value <- new_value
);

let int_mutator = create_escaping_ref 42;
let str_mutator = create_escaping_ref "hello";

// These should work since the type is fixed at creation
print int_mutator 100;
print str_mutator "world";

// But what if we try to assign one mutator to another?
let mixed_mutator = if true then int_mutator else str_mutator;