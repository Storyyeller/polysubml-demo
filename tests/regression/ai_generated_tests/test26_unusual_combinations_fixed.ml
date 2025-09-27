// Test 26: Unusual combinations (fixed)
// Test mixing tuples with variants
let weird = `Data (1, "hello", true);
match weird with
| `Data (a, b, c) -> (print a, b, c);

// Test deeply nested records
let nested = {
    level1 = {
        level2 = {
            level3 = {
                value = 42
            }
        }
    }
};
print nested.level1.level2.level3.value;

// Test mutable fields in nested records
let mut_nested = {
    mut container = {
        mut count = 0;
        name = "test"
    }
};
print mut_nested.container;
mut_nested.container <- {mut count = 5; name = "updated"};
print mut_nested.container;