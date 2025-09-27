// Test case: let rec with duplicate bindings
// Should produce error about duplicate variable names

let rec
  f = fun x -> x + 1
and
  f = fun y -> y * 2
in
f 5