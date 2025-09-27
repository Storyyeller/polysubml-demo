// Test 11: Basic variant types and pattern matching
let shape1 = `Circle {radius=5.0};
let shape2 = `Rectangle {width=3.0; height=4.0};
let shape3 = `Square {side=2.0};

let area = fun shape ->
    match shape with
    | `Circle {radius} -> radius *. radius *. 3.14159
    | `Rectangle {width; height} -> width *. height
    | `Square {side} -> side *. side;

print area shape1;
print area shape2;
print area shape3;