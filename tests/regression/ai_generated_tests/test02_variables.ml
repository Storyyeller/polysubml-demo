// Test 2: Variable bindings and conditionals
let x = 42;
let y = if x > 40 then "big" else "small";
print x, y;

let z = (
    let a = 5;
    let b = a * 2;
    a + b
);
print z;