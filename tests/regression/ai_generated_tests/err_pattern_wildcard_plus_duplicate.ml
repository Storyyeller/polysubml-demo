// Test case: wildcard match with duplicate specific pattern
// Should catch multiple handlers for same variant

match `A 5 with
  | `A x -> x
  | _ -> 0
  | `A y -> y + 1  // Duplicate after wildcard