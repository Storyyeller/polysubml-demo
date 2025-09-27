// Test case: match expression with duplicate match tags
// Should produce error about duplicate pattern matching tags

match `A 5 with
  | `A x -> x
  | `B s -> 0
  | `A y -> y + 1