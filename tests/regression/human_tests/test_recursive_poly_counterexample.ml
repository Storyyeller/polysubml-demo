let _ = fun x -> (
  let j: type A. any -> any -> any -> A = x.j;
  let k = {
    mut x: type A. any -> type A as B. any -> rec u=any -> A | B | u = j
  };
  // This line compiles fine due to the explicit type annotation.
  k.x <- (fun x -> j: any -> type A. any -> any -> any -> A);
  // The same line is incorrectly rejected when the type annotation is removed
  // due to a limitation in the typechecker algorithm.
  k.x <- (fun x -> j);

  x 
)