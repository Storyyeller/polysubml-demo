// Test 28: Syntax edge cases (fixed)
// Test multiple let bindings without trailing expression at top level
let a = 5;
let b = 10;
let c = a + b;

// Test empty string and unusual string content
print "";
print "\"quotes\" and 'apostrophes'";

// Test various number formats
print 0;
print -42;
print 1.0e10;  // Fixed: must have decimal point
print -3.14e-2;