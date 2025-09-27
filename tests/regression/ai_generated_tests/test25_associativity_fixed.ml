// Test 25: Function application associativity (fixed)
// Test that a b c is parsed as a (b c), not (a b) c

let f = fun x -> x + 10;
let g = fun x -> x * 2;

// This should parse as f (g 5) = f (10) = 20, not (f g) 5
print f g 5;

// Test with explicit parentheses to verify
print f (g 5);  // Should be same as above
print (f g) 5;  // This should cause a type error since f g tries to apply f to g