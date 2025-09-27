// Test 14: Polymorphic functions
let id = fun (type t) (x: t): t -> x;

print id 42;
print id "hello";
print id 3.14;

let swap = fun (type a b) (x: a, y: b): b * a -> (y, x);
print swap ("first", "second");
print swap (1, 2.5);