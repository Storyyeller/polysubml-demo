// Test 52: Complex combination of mutation, polymorphism, and existential types
// Create a polymorphic mutable container with existential operations
let create_complex_container = fun (type t) (initial: t) -> (
    let storage = {mut value = initial; mut count = 0};
    let incrementer = fun _ -> storage.count <- storage.count + 1;
    {storage; inc = incrementer; get_stats = fun _ -> (storage.value, storage.count)}
);

let int_container = create_complex_container 42;
let str_container = create_complex_container "hello";

// Use them
int_container.inc {};
int_container.inc {};
str_container.inc {};

print int_container.get_stats {};
print str_container.get_stats {};

// Mix them via conditional - this should create existential types
let mixed = if true then int_container else str_container;
mixed.inc {};

// What can we learn about the mixed container?
let stats = mixed.get_stats {};
print stats;