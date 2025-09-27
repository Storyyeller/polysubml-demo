// Test 22: Empty and single-element records
let empty = {};
print empty;

let single = {_0="only"};
print single;
print single._0;

// Test destructuring empty record
let {} = empty;
print "empty destructured";

// Test one-element tuple syntax
let {_0=x} = single;
print x;