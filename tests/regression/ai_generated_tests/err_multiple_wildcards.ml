// Test case: multiple wildcard patterns in match
// Should produce error about multiple wildcards

match `A 5 with
  | `A x -> x
  | _ -> 0
  | _ -> 1  // Second wildcard should error