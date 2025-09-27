// Test 21: Edge cases with strings and floats
print "test" ^ "";
print "" ^ "test";
print "" ^ "";

// Test floating point edge cases
print 1.0 /. 0.0;
print -1.0 /. 0.0;
print 0.0 /. 0.0;

// Test large numbers
print 999999999999999999999;
print 1.7976931348623157e308;