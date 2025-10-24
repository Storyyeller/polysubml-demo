let f = fun x -> (
  let a = {};
  x
);
let g = fun x -> (
  let a = {x};
  {a; x}
);