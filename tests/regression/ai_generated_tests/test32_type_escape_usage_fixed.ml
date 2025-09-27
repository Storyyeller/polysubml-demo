// Test 32: Using escaped type variables (fixed)
let create_container = fun (type t) (x: t) -> (
    let getter = fun _ -> x;
    {value = x; get = getter}
);

let container1 = create_container 42;
let container2 = create_container "hello";

// Test if we can access the values
print container1.value;
print container2.value;
print (container1.get {});
print (container2.get {});

// Test if containers have compatible types by assigning one to another
let test_compat = container1;
let test_compat2 = container2;