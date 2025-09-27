// Test 10: Tuples and record equivalence
let tup = 1, "hello", 3.14;
print tup;

// Access tuple fields both ways
print tup._0;
print tup._1;
print tup._2;

// Destructure tuple
let (a, b, c) = tup;
print a, b, c;

// Mix tuple and record syntax
let mixed = {_0=10; name="test"; _1=20; _2=30};
print mixed;

let (x, y, z) = mixed;
print x, y, z;
print mixed.name;