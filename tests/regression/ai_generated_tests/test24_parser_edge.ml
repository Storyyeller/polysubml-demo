// Test 24: Parser edge cases
// Test different comment styles
(* block comment *)
// line comment
let x = 42; (* inline block *)
let y = x + 1; // inline line

// Test whitespace sensitivity
let   a   =   5   +   3  ;
let b=6+2;
print a, b;

// Test begin/end vs parentheses
let result1 = (
    let temp = 5;
    temp * 2
);

let result2 = begin
    let temp = 7;
    temp * 3
end;

print result1, result2;