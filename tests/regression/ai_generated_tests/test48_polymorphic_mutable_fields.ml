// Test 48: Mutable fields in polymorphic contexts
// Can we create a polymorphic function that works with mutable fields?
let swap_mutable = fun (type t) (r1: {mut a: t; b: t}, r2: {mut a: t; b: t}) -> (
    let temp = r1.a;
    r1.a <- r2.a;
    r2.a <- temp
);

let record1 = {mut a = 10; b = 20};
let record2 = {mut a = 30; b = 40};

print record1, record2;
swap_mutable (record1, record2);
print record1, record2;