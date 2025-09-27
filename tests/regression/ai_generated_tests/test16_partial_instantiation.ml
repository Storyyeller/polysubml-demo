// Test 16: Partial instantiation error case
let make_pair = fun (type a) (x: a): (type b. b -> a * b) ->
  fun (type b) (y: b): a * b ->
    (x, y);

// This should fail according to the README
let pair = (make_pair 123) "bar";