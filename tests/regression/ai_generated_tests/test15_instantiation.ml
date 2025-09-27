// Test 15: Explicit instantiation
let id = fun (type t) (x: t): t -> x;

let id_int: int -> int = id[t=int];
let id_str: str -> str = id[t=str];

print id_int 42;
print id_str "test";

// Test instantiation with empty brackets
let id_auto = id[];
print id_auto 99;