print (let rec f = fun x -> f x in 42);

let rec f = fun (x, n) -> if x == 1 then n else g (x, n)
and g = fun (x, n) -> (
    if x % 2 == 0 then f (x / 2, n + 1)
    else h (x, n + 1)
)
and h = fun (x, n) -> f (3*x+1, n)
  in f (27, 0)