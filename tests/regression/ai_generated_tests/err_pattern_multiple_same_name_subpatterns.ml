// Test case: pattern that binds multiple variables with the same name
// where conflicting bindings appear in distinct subpatterns
// Should produce error about variable being bound multiple times

match `Left 5 with
  | `Left x -> x
  | `Right x -> x

// This should be valid, but let's test a case where it's invalid:
let test_tuple = match (1, 2) with
  | (x, x) -> x  // This should error - x bound twice in same pattern