let add3 = fun x -> x + 3;
let mul10 = fun x -> x * 10;
print mul10 7 |> add3 |> mul10;

