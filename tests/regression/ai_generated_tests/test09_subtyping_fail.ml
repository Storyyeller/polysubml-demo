// Test 9: Structural subtyping failure case
let get_record = fun flag ->
    if flag then
        {a=1; b=2; c=3}
    else
        {a=4; b=5; d=6};

let r = get_record true;
// This should fail - c is not available in both branches
print r.c;