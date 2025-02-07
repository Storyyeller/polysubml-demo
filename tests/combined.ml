### Good
let f = fun x -> x[];

// Test recovery from bad universal instantiation
### Bad
1 + (f (fun (type t) (x: t) : t -> x)) 1;
x + 4;

### Good
1.0 +. (f (fun (type t) (x: t) : t -> x)) 1.0;

let r = {a=1; b=fun x->x+1; c=1.2; d=fun x->x+.2.1};

// Test recovery from bad existential instantiation
### Bad
let {type t; a: t; b: t->t} = r;
4 +. 3;

### Good
let {type t; c: t; d: t->t} = r;

fun (type t) (x: t, f: type u. t * u->int * u) : int * t -> (
  let (a, b) = f (x, 23);
  let (c, d) = f (x, {x=a+b});
  c + d.x;

  f (x, x)
);

let rec fizzbuzz = fun i -> (
  print (if i % 3 == 0 then
    if i % 5 == 0 then 
      "FizzBuzz"
    else      
      "Fizz"
  else 
    if i % 5 == 0 then
      "Buzz"
    else 
      i 
  );
  if i < 50 then 
    fizzbuzz (i+1)
  else 
    0
);
fizzbuzz 0;

let vars = {mut i=0};
loop if vars.i >= 50 then `Break 0 else (
  let i = vars.i <- vars.i + 1;
  print (if i % 3 == 0 then
    if i % 5 == 0 then 
      "FizzBuzz"
    else      
      "Fizz"
  else 
    if i % 5 == 0 then
      "Buzz"
    else 
      i 
  );
  `Continue 0
);

let x = (
    let a = 4; 
    let b = a + 3; 
    let c = a * b; 
    let d = c / b;
    d + a
);

let x = (
  print "test", 1+2, 3 * 9, 2.0 /. 0.0;
  42
);

let (a: int, b: str, c, d: float) = 1, "", "", 3.2;

let {a; b: int; c=x: int; d={e: float}} = {a=1; b=2; c=3; d={e=4.4}};

match `Foo {x=32} with
| `Foo {x: int} -> x;

let add = fun {a: int; b: int} -> a + b;
match `Foo 32 with
| `Foo (x: int) -> x;

let add_one = fun (x: int) -> x + 1;
let a = 42;
let b = -9.7;

let r = {
  a: int;
  x: int = a;
  mut b: float;
  mut y: float = b
};

let add_curried = fun (a: int) : (int -> int) -> 
  fun (b: int) : int -> a + b;

print (add_curried 4) 22; // 26;

(`Foo 3: [`Foo of int | `Bar float]);











let x = {mut v=`None 0; mut t=`None 0};
x.v <- `Some (x.v, {a=0; b=fun x->x+1});
x.v <- `Some (x.v, {a=0.2; b=fun x->x+.9.1});
x.v <- `Some (x.v, {a={q=1}; b=fun {q}->{q}});

loop match x.v with 
| `None _ -> `Break 0
| `Some (t, h) -> (
  x.v <- t;

  let {type t; a: t; b: t->t} = h;
  print (match x.t <- `Some a with 
  | `None _ -> "missing"
  | `Some t -> t
  );

  `Continue 0
);



### Bad
let f = fun (type t) (x: t) : t -> x;
1 + f 3.2;

### Bad
// Test for types escaping loops
let x = {mut v=`None 0; mut t=`None 0};
x.v <- `Some (x.v, {a=0; b=fun x->x+1});
x.v <- `Some (x.v, {a=0.2; b=fun x->x+.9.1});
x.v <- `Some (x.v, {a={q=1}; b=fun {q}->{q}});

loop match x.v with 
| `None _ -> `Break 0
| `Some (t, h) -> (
  x.v <- t;

  let {type t; a: t; b: t->t} = h;
  print (match x.t <- `Some a with 
  | `None _ -> "missing"
  | `Some t -> b t
  );

  `Continue 0
);



### Bad
let a = 3;
{a: any}.a + 1;
