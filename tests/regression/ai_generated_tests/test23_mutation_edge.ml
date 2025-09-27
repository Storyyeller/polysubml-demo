// Test 23: Mutation assignment edge cases
let obj = {mut a=10; mut b=20; mut c=30};
print obj;

// Test that assignment returns old value
let old_a = obj.a <- 100;
print "old a:", old_a;
print "new obj:", obj;

// Test chained assignments (should work right-to-left)
obj.a <- obj.b <- obj.c <- 999;
print obj;

// Test self-referential assignment
obj.a <- obj.a + 1;
print obj;