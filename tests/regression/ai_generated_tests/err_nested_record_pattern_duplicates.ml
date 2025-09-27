// Test case: deeply nested record patterns with duplicate bindings
// Should catch binding conflicts in nested destructuring

match {a = {b = {c = 1}}} with
  | {a = {b = {c = x}; b = y}} -> x + y  // 'b' appears twice