// Test 18: Explicit existential instantiation
let r = {type t=int; type u=int * int;
  a=3; f=fun x->(x, x+1); g=fun (x, y) -> x * y};

let {type t; type u; a: t; f: t->u; g: u->t} = r;
print g f a;