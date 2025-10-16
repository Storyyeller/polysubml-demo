let _ = fun x -> (
  let d: rec t=t * (type Q as E Q as D. D -> E) = (x.d : rec t=t * (type Q as R Q. R -> Q));
 
  let e: type A. A -> type B. B -> A | B | int = (x.e : type A. A -> type B. B -> A | B | int);
  let f: type A. A -> type B. B -> rec t={x:A | B | t} = (x.f : type A. A -> type B. B -> rec t={x:A | B | t});
  let g: type A. A -> type B. B -> rec t={x:A | B; y:t} = (x.g : type A. A -> type B. B -> {x: A; y: {x: B; y: never}});
  let h: type A. A -> type A as B. B -> rec t={x:A | B; y:t} = (x.h : type A. A -> type A as B. B -> {x: A; y: {x: B; y: never}});


 x 
)