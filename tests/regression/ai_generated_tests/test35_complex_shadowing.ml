// Test 35: Complex type variable shadowing scenarios
let multi_level = fun (type a) (x: a) ->
    fun (type b) (y: b) ->
        fun (type a) (z: a) -> (  // Shadow the outer 'a'
            // Which 'a' does this refer to?
            let result: a = z;
            // Can we access the outer 'a' via x? This should fail
            let bad: a = x;
            result
        );