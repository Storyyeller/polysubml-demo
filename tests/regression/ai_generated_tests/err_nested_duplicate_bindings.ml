// Test case: nested patterns with duplicate bindings across levels
// Should catch complex scoping errors

match `Node (`Leaf 1, `Leaf 2) with
  | `Node (`Leaf x, `Node (`Leaf x, _)) -> x  // x bound in different subpatterns