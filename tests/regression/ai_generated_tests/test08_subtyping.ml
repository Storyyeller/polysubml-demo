// Test 8: Structural subtyping
let person = {name="Alice"; age=30; city="NYC"};
let minimal = {name="Bob"; age=25};

// Function that only needs name and age
let greet_person = fun p -> "Hello " ^ p.name ^ ", age " ^ (
    if p.age < 18 then "minor" else "adult"
);

print greet_person person;
print greet_person minimal;

// Test with conditional that creates different record types
let get_record = fun flag ->
    if flag then
        {a=1; b=2; c=3}
    else
        {a=4; b=5; d=6};

let r = get_record true;
print r.a;
print r.b;