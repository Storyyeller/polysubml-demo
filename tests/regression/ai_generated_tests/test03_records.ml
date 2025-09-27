// Test 3: Records and field access
let person = {name="Alice"; age=30; active=true};
print person.name;
print person.age;

// Test destructuring
let {name; age} = person;
print "Name:", name, "Age:", age;

// Test mutable fields
let counter = {mut count=0; label="test"};
print counter;
counter.count <- counter.count + 1;
print counter;