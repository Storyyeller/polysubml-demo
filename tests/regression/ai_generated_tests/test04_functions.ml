// Test 4: Functions
let add = fun (a, b) -> a + b;
print add (3, 5);

let greet = fun {name; formal=f} ->
    if f then "Hello, " ^ name else "Hi " ^ name;
print greet {name="Bob"; formal=true};
print greet {name="Alice"; formal=false};

// Test curried function
let multiply = fun x -> fun y -> x * y;
print (multiply 4) 6;