// Test case: attempting to mutate immutable field
// Should produce error about field not being mutable

let r = {x = 1; y = 2};
r.x <- 5