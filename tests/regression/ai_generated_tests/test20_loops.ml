// Test 20: Loop expressions
let sum_to_n = fun n -> (
    let state = {mut i=0; mut sum=0};
    loop if state.i >= n then `Break state.sum else (
        state.i <- state.i + 1;
        state.sum <- state.sum + state.i;
        `Continue 0
    )
);

print sum_to_n 5;
print sum_to_n 10;