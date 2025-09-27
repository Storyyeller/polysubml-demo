// Test 25: Function application associativity (simple)
let f = fun x -> x + 10;
let g = fun x -> x * 2;

// These should be equivalent due to right associativity
print f g 5;      // f (g 5) = f 10 = 20
print f (g 5);    // f 10 = 20