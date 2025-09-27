// Test case: loop with incorrect return type
// Should catch type errors in loop body

let vars = {mut i = 0};
loop if vars.i > 5 then
  42  // Should return `Break something, not raw int
else (
  vars.i <- vars.i + 1;
  `Continue 0
)