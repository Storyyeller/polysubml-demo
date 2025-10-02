let f = fun (type t) (x: t): t -> x;
let g = fun {}: (type t. t -> t) -> f;
let _: str = 42 |> (g {});
