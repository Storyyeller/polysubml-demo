let f = fun (type t) (x: t): t -> x;
let _: str = 42 |> f[t=int];
