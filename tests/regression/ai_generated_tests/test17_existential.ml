// Test 17: Existential types
let r = if false then
  {a=3; b=fun x->x+1}
else
  {a=9.1; b=fun x->x*.2.0};

let {type t; a: t; b: t->t} = r;
print b b a;