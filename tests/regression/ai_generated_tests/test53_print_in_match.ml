// Test 53: Correct way to use print in match expressions
let test_value = `Circle {radius=5.0};

let result = match test_value with
| `Circle {radius} -> (print "found circle with radius", radius; radius *. radius *. 3.14159)
| `Square {side} -> (print "found square with side", side; side *. side);

print "area:", result;