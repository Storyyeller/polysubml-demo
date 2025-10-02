PolySubML is an experimental ML-like programming language with the following features:

* Structural subtyping (aka "compile-time duck typing")
* Global type inference (no type annotations required at all for monomorphic code)
* Polymorphic functions
* Higher rank types
* Existential types
* Equirecursive types
* No unnameable or [inconceivable types](https://blog.polybdenum.com/2024/06/07/the-inconceivable-types-of-rust-how-to-make-self-borrows-safe.html)
* **Worst-case polynomial time type checking**

You can try it out online in your browser [here](https://storyyeller.github.io/polysubml-demo/demo.html). PolySubML is not intended to be used as a production language, but rather serves as a demonstration of a novel type inference algorithm and other language design techniques.

### Example

Here's a FizzBuzz implementation using a recursive function:

```ocaml
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
```

And here's an imperative FizzBuzz implementation using a loop and mutable counter:

```ocaml
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
```

## Getting started

You can try out PolySubML online in your browser at https://storyyeller.github.io/polysubml-demo/demo.html. To build it from source, install `lalrpop` and `wasm-pack` and then run

```
lalrpop src/grammar.lalr
wasm-pack build --target web --no-typescript wasm --out-dir ../pkg
```

## A quick tour of PolySubML

PolySubML supports both `(* style *)` and `// style` comments. `//` comments continue until the end of the line. Whitespace is otherwise completely insignificant, except that tokens can be separated by one or more whitespace characters to avoid ambiguity (e.g. `a b` is parsed as two tokens while `ab` is one token), but the kind and amount of whitespace used does not matter.

#### Primitives and operators

PolySubML has the primitive types `bool` (`true` or `false`), `int` (arbitrary precision integers), `float` (64 bit floating point), and `str` (strings).

The integer operators are `+`, `-`, `*`, `/`, `%`, `<`, `<=`, `>`, and `>=`. For floating point operations, suffix the operator with `.`, e.g. `1.1 +. 2.2`. The equality operators `==` and `!=` accept values of any type, but different types compare nonequal. String concatenation is `^`.

```
>> 5 + 77
82
>> 8.1 *. 1.1
8.91
>> "Hello, " ^ " World!"
"Hello,  World!"
>> 7 < -99
false
>> -9.9 <. 1242.1e3
true
>> 9 == 9.0
false
>> 10 == 10
true
>> 5.4 != "5.4"
true
```

For non-primitive values, equality comparison is by object identity:

```
>> let a = {}
{}
>> a == a
true
>> a == {}
false
```


#### Expressions and statements

PolySubML is an expression oriented language where nearly everything is an expression, including conditionals, loops, function definitions, and more. However, it also has statement-like syntax.

In OCaml, the basic assignment expression is `let v = expr1 in expr2`, where variable `v` is in scope for `expr2`. This works all right in simple cases, but becomes annoying when you have a long sequence of assignments (e.g. `let a = 4 in let b = a + 3 in let c = a * b in let d = c / b in d + a`). 

PolySubML supports the traditional `let v = expr1 in expr2` syntax, but it *also* allows you to use a `;` in place of `in`, e.g. `let v = expr1; expr2`. The two are exactly equivalent, except for parsing precedence (`;` has the lowest precedence, so you'll usually have to surround the block in parenthesis). This is much more convenient when chaining assignments and allows you to write imperative-style code like this:

```ocaml
let x = (
    let a = 4; 
    let b = a + 3; 
    let c = a * b; 
    let d = c / b;
    d + a
);
```

> *You can also group expressions using `begin ... end` instead of `( ... )` if you want.*

Note that everything is still an expression, which means that these "statement block" expressions still have to end in an expression. For example, `(let a = 3; let b = 3;)` is not valid because there's no trailing expression in the block. The exception is at the top level of a source file. At the top level, a source file consists of zero or more statements with no trailing expression.

For convenience, PolySubML also includes a print "statement", which prints one or more comma separated expressions to the console:

```
let x = (
  print "test", 1+2, 3 * 9, 2.0 /. 0.0;
  42
);
```


#### Conditionals

In PolySubML `if` is an expression, not a statement. The general form is `if <expr> then <expr> else <expr>`. For example, evaluating `if false then "Hello" else "World"` would result in `"World"`. You can think of this as similar to the ternary operator (`a ? b : c`) in C-style programming languages.

The `else` part of `if` expressions is mandatory. When the else branch contains "statements", it has to be wrapped in parenthesis (or `begin`/`end`) to avoid ambiguity.

#### Records and fields

Records are a grouping of zero or more named values similar to "objects" or "structs" in other programming languages and are defined via `{name1=val1; name2=val2; ...}`. You can access the value of a field using the usual `.name` syntax. For example `{a=true; b="Hello"; c={}}.b` evaluates to `"Hello"`.

There is a special shorthand syntax for fields with the same name as their value - `{a; b; c=4}` is equivalent to `{a=a; b=b; c=4}`.

Unlike in OCaml, records in PolySubML are anonymous and structurally typed. A record with more fields is a subtype of a record with fewer fields, leading to a "compile time duck typing" effect.

For example, this code typechecks even though the branches of the if expression are records with different sets of fields. 

```ocaml
let x = if true then 
        {a=4; b=2} 
    else 
        {c=9; a=1};
x.a
```

However, everything is still statically typed. Attempting to access a possibly undefined field results in a compile error:

```ocaml
let x = if true then 
        {a=4; b=2} 
    else 
        {c=9; a=1};
let _ = x.b;
```
```
TypeError: Missing field b
Note: Field b is accessed here:
    else 
        {c=9; a=1};
let _ = x.b;
         ^~  
But the record is defined without that field here:
        {a=4; b=2} 
    else 
        {c=9; a=1};
        ^~~~~~~~~~  
let _ = x.b;
Hint: To narrow down the cause of the type mismatch, consider adding an explicit type annotation here:
let x: _ = if true then 
     +++                 
        {a=4; b=2} 
    else
```

#### Destructuring and let patterns

In order to more conveniently access multiple fields of a record value, you can *destructure* records when assigning them. For example, instead of doing `let foo = record.field1; let bar = record.field2; let baz = record.field3` etc., you can just write `let {field1=foo; field2=bar; field3=baz} = record;` instead.

As with record value construction, you can omit the `=` part when the field and variable name match. For example, `let {field1=foo} = record` is equivalent to `let foo = record.field1`, while `let {foo} = record` is equivalent to `let foo = record.foo`.

You can mix and match the shorthand and full field assignment syntax:

```ocaml
let x = {a=2; b=7; c=9};

let {c; a=foo} = x;
print foo, c; // 2 9
```

Additionally, record destructuring can be nested. The left hand side of an assignment is actually a *pattern*, not just a variable name. The simplest kind of pattern is a *variable pattern*, which just binds a single variable (e.g. `let a = ...` binds the variable `a`), but patterns can also be *record patterns* as described above. 

When using the `{field=pattern}` syntax in a record pattern, the right hand side can be *any pattern*, including variable or record patterns, meaning that patterns can be arbitrarily nested:


```ocaml
let record = {a=3; b=4; c={d=5; e=6; f={a=90}}};
let {a; b=x; c={d; e=y; f={a=z}}} = record;
print a, x, d, y, z; // 3 4 5 6 90
```


#### Mutable fields

In PolySubML, variables cannot be reassigned, merely shadowed. The *only* way to have mutable state is via *mutable record fields*. When creating a record value, you can optionally make fields mutable by prefixing them with `mut` and you can update the value of mutable fields via `record.name <- new_value`:


```ocaml
let x = {a=4; mut b=6; c=9};
print x; // {a=4; b=6; c=9}
x.b <- x.b + 11;
print x; // {a=4; b=17; c=9}
```

Records use reference semantics, so mutating a field affects all references to the same object, even if they are bound to different variable names:


```ocaml
let x = {mut i=1};
let y = x;
let z = {mut i=1};
print x, y, z; // {i=1} {i=1} {i=1}

y.i <- 32;
print x, y, z; // {i=32} {i=32} {i=1}
```

Record field assignment is itself an expression. Unlike C-family languages, where assignment expressions evaluate to the *new* value and unlike Rust and ML-family languages where it evaluates to `()`, in PolySubML, assignment evaluates to the *old* value (the one being replaced). This is generally a lot more useful, since it lets you easily swap values:


```ocaml
let x = {mut a=3; mut b=8};
print x; // {a=3; b=8}

x.a <- x.b <- x.a;
print x; // {a=8; b=3}
```


#### Tuples

You can create *tuples* via `a, b, c, ...` and likewise destructure them with `(a, b, c, ...)`.

```ocaml
let tup = 42, true, "hello", 9.1;
let (a, b, c) = tup;

print a; // 42
print b; // true
print c; /// hello
```

You can also access specific fields of a tuple via `._0`, `._1`, etc. For example `(1, 3, 5, 7, 9)._3` is `7`. In fact **tuples are just a special case of records**. The expression `a, b, c` is just a shorthand syntax for the record expression `{_0=a; _1=b; _2=c}`, and the pattern `(a, b, c)` is just a shorthand syntax for the record pattern `{_0=a; _1=b; _2=c}`. Since tuple values are just record values, you can freely mix and match the shorthand and explicit syntax, and you can also combine tuple fields and other named fields:


```ocaml
let {_0=a; _1=b; _2=c} = 6, 7, 8;
let foo = {_2=c; blah="hello"; _1="world"; a; _0=9};
print foo; // {_2=8; blah="hello"; _1="world"; a=6; _0=9}

let (x, y, z) = foo;
print x, y, z; // 9 world 8
```

There is no shorthand syntax for 0 or 1 element tuples. For that, you have to use the explicit record syntax (`{}` or `{_0=x}` respectively). Furthermore, tuple syntax does not support mutable fields or existential types. If you want to use those, you need to use the explicit record syntax.


```ocaml
let foo = {_4=true; _2="a"; mut _1="b"; _0="c"; _3=42};
print foo; // ("c", "b", "a", 42, true)

foo._1 <- 9.7;
print foo; // ("c", 9.7, "a", 42, true)
```

Variable patterns with the name `_` do not bind a variable, and instead just ignore the matched value. This is mainly useful in tuple patterns when you don't care about the middle values:


```ocaml
let tup = 42, true, "hello", 9.1;
let (a, _, _, c) = tup;

print a; // 42
print c; /// 9.1
```



#### Functions

In PolySubML, all functions are required to take exactly one argument for simplicity. They are defined by `fun <arg> -> <expr>`. For example, the identity function is written `fun x -> x`. Functions are called by simply suffixing an argument, i.e. writing `a b` where `a` is the function to be called and `b` is the argument. For example 

    (fun b -> if b then "Hello" else "World") true

evaluates to `"Hello"`, while 

    (fun x -> x.foo) {bar=false; foo="Bob"}

evaluates to `"Bob"`. 

> Note: Unlike in OCaml, function application is right-associative rather than left-associative. In other words, `a b c` is parsed as `a (b c)`, rather than `(a b) c` like it would be in OCaml.

PolySubML also supports the reverse application operator `|>`. `a |> b` is equivalent to `b a` except that `a` will be evaluated before `b`. Additionally, the `|>` operator is left-associative, meaning that `a |> b |> c` parses as `(a |> b) |> c`, which is in turn equivalent to `c b a`.


#### Multiple arguments

In order to define functions that take multiple arguments, you can just use a tuple as the function argument instead:

```ocaml
let sub = fun (a, b) -> a - b;
print sub (2, 5); // -3
```

You can also simulate named arguments by taking a record instead:

```ocaml
let sub = fun {a; b} -> a - b;
print sub {a=2; b=5}; // -3
print sub {b=5; a=2}; // -3
```

In fact, the function argument can be any pattern, so you can even nest patterns if you want:
```ocaml
let f = fun {a; b={c=x; d}} -> a + x * d;
print f {b={c=4; d=7}; a=12}; // 40
```

An alternative approach to multiple argument functions is *currying*, where the original function just returns another function until all arguments have been applied:

```ocaml
let sub = fun a -> fun b -> a - b;
print (sub 2) 5; // -3
```

However, currying is not recommended because it leads to confusing type error messages if you mess up the arguments, and it interacts poorly with polymorphism. Furthermore, you can't simulate named arguments the way you can if you take a record as the function argument.


#### Recursive let bindings

Sometimes, one wishes to have functions that call themselves recursively. Unfortunately, this is impossible with the above constructs since plain let-expressions can only refer to variables that were already defined. 

In order to support recursion, PolySubML offers _recursive let expressions_ which are defined via `let rec` and allow the definition of the variable to refer to itself. For example, you could define a recursive fibonacci function as follows:

```ocaml
let rec fib = fun x ->
    if x <= 1 then 
        1
    else
        fib(x - 1) + fib(x - 2)
```

In order to avoid code referring to variables that don't exist yet, the right hand side of `let rec` variable definitions is restricted to be a function definition.


#### Mutual recursion

The above syntax works for a single function that refers to itself, but in some cases, you may want to have multiple functions that each refer to each other. Unlike in the case with `let`, simply nesting `let rec`s won't work. Therefore, `let rec` allows _multiple_ variable bindings, separated by `and`. For example, you can define mutually recursive `even` and `odd` functions as follows:

```ocaml
let rec even = fun x -> if x == 0 then true else odd(x - 1)
    and odd = fun x -> if x == 0 then false else even(x - 1)
```


#### Variants and matching

Sometimes you need to make different decisions based on runtime data in a type safe manner. PolySubML supports this via _variants_, also known as _sum types_ or _enums_. Basically, the way they work is that you can wrap a value with a tag, and then later match against it. The match expression has branches that execute different code depending on the runtime value of the tag. Crucially, each match branch has access to the static type of the original wrapped value for that specific tag.

To wrap a value, prefix it with a grave (\`) character and an identifier tag, e.g. `` `Foo 42``.

As with records, you can destructure variant values using variant patterns:

```ocaml
let x = `Foo {a=42};
let `Foo {a} = x;
print a; // 42
```

However, this isn't terribly useful, since it is limited to variant values that statically have only one possible variant. In order to do anything useful with variants, you need to use a *match expression*.

Match expressions list one or more possible variants, and an expression to execute in each case. For example:

```ocaml
let calculate_area = fun shape ->
    match shape with
        | `Circle {rad} -> rad *. rad *. 3.1415926
        | `Rectangle {length; height} -> length *. height;

calculate_area `Circle {rad=6.7};
calculate_area `Rectangle {height=1.1; length=2.2};
```

Notice that within the Circle branch, the code can access the rad field, and within the Rectangle branch, it can access the length and height field. Variants and matches let you essentially "unmix" distinct data types after they are mixed together in the program flow. Without variants, this would be impossible to do in a type-safe manner.




#### Wildcard matches

The above match expressions are *exclusive*, meaning that any unhandled vairant results in a compile time error. You can optionally instead include a *wildcard* arm, which will match any variants not matched by the other match arms.


```ocaml
let calculate_area = fun shape ->
    match shape with
        | `Circle {rad} -> rad *. rad *. 3.1415926
        | `Rectangle {length; height} -> length *. height
        |  v -> "got something unexpected!"
```

Within a wildcard match, the bound variable has the same type as the input expression, except with the explicitly matched cases statically excluded. For example, in the `calculate_area` example above, `v` would have the type "same as `shape` except not a `Circle` or `Rectangle`".

This makes it possible to further match on the wildcard value elsewhere. For example, in the below code, the new `calculate_area2` function explicitly handles the `Square` case and otherwise defers to the previously defined function to handle the `Circle` and `Rectangle` cases. This works because the compiler knows that the `v` in the wildcard branch is not a `Square`, so it will not complain that the original `calculate_area` function fails to handle squares.

```ocaml
let calculate_area = fun shape ->
    match shape with
        | `Circle {rad} -> rad *. rad *. 3.1415926
        | `Rectangle {length; height} -> length *. height;

let calculate_area2 = fun shape ->
    match shape with
        | `Square {len} -> len *. len
        |  v -> calculate_area v;

calculate_area2 `Circle {rad=6.7};
calculate_area2 `Square {len=9.17};
```

#### Match ordering

PolySubML requires match expressions to be *exhaustive*, meaning that any unhandled variants result in a compile time error (unless you include a wildcard match). 

Furthermore, PolySubML **matches are order-independent and match exactly one level of tags** per match expression. For example, consider this code:

```ocaml
let x = `Foo `NotBar 0;

match x with
| `Foo `Bar _ -> "hello"
| _ -> "world";
```

In languages with order-dependent match expressions, this would evaluate to "world", because the pattern matching goes arbitrarily deep. Even though the `Foo` matches the value, the `Bar` doesn't, so it just silently falls through to the wildcard case.

In PolySubML by contrast, the choice of match arm depends *only* on the top-most tag. This means that the `Foo` branch will be taken, resulting in a compile error due to the exhaustive `Bar` pattern:

```
TypeError: Unhandled variant NotBar
Note: Value originates here:
let x = `Foo `NotBar 0;
             ^~~~~~~    
match x with
But it is not handled here:
match x with
| `Foo `Bar _ -> "hello"
       ^~~~              
| _ -> "world";
Hint: To narrow down the cause of the type mismatch, consider adding an explicit type annotation here:
let x: _ = `Foo `NotBar 0;
     +++                   
match x with
```

As another example, consider this code:

```ocaml
let x = `Foo 0;

match x with
| _ -> "world"
| `Foo _ -> "hello"
```

In languages with order-dependent match expressions, this would evaluate to "world" because everything matches the wildcard and all other arms are unreachable. However, in PolySubML, matching is order independent, so this evaluates to "hello" instead. Additionally, in PolySubML, it is a compile error if the same variant is matched twice in a match expression, or if there are multiple wildcards.







### Type annotations

PolySubML's powerful type inference means that no type annotations are required at all except for polymorphic code. However, even when type annotations are not necessary, it can be helpful to add type annotations in order to clarify intent in the code and help narrow down the scope of type errors when there is a compile error. 

#### Where can I put type annotations?

First off, expressions can manually be annotated with a type via `(expr : type)`, e.g. `(24 : int)` or `(fun x -> x : str -> str)`.

Second, type annotations can be added to any variable pattern, e.g. `let a: int = 43;`. This applies to anywhere that variable patterns can appear, including `let`, `match` arms, function arguments, and nested in other patterns. 

```ocaml
let (a: int, b: str, c, d: float) = 1, "", "", 3.2;

let {a; b: int; c=x: int; d={e: float}} = {a=1; b=2; c=3; d={e=4.4}};

match `Foo {x=32} with
| `Foo {x: int} -> x;

let add = fun {a: int; b: int} -> a + b;
```

You can even add type annotations to `_` variable patterns, e.g. (`let _: int = 42;`). They don't actually bind a variable, but the type constraint will still be applied.

When adding a type annotation to a bare variable pattern in a match arm or function argument, it has to be surrounded in parenthesis:

```ocaml
match `Foo 32 with
| `Foo (x: int) -> x;

let add_one = fun (x: int) -> x + 1;
```

Third, you can add type annotations to fields in a record literal:

```ocaml
let a = 42;
let b = -9.7;

let r = {
  a: int;
  x: int = a;
  mut b: float;
  mut y: float = b
};
```

Lastly, you can also annotate the return type of a function definition. This goes after the argument pattern and before the `->`:

```ocaml
let add = fun {a: int; b: int} : int -> a + b;
```

> :warning: **Warning:** In `fun x: int -> ...`, the `int` annotations the *return type* of the function, not the argument. To annotate the argument, you need to surround it in parenthesis. For example, `fun (x: float) : int -> ...` is a function which takes a `float` and returns an `int`.

When the return type is itself a function type (e.g. `int -> int`), it has to be surrounded in parenthesis. For example, here's how to correctly annotate all the arguments and return types of a curried function:


```ocaml
let add_curried = fun (a: int) : (int -> int) -> 
    fun (b: int) : int -> a + b;

print (add_curried 4) 22; // 26
```

This is necessary to avoid ambiguity with the `->` indicating the function body. OCaml requires types and expressions to use different capitalization so they are syntactically distinct and there is no ambiguity. However, PolySubML has no such restriction on capitalization, so parenthesis are needed to disambiguate this specific case.

#### How do I write type annotations?

* Basic types: `bool`, `float`, `int`, `str`, `any`, and `never`

`any` is a special type that can hold *any* value. `never` is a special type that can never hold any value, and thus it is impossible to create a value of type `never`. Any code with such a value is thus guaranteed to be unreachable.

Note that this differs from the "any" type of some type checkers, where it is used as a deliberately unsound escape hatch from the type system. In PolySubML, `any` is 100% sound and type safe. It can hold any value, but this means you can't actually do anything with values of `any` type other than equality comparisons.

* Function types: `int -> int`

* Record types: `{field1: int; field2: str; mut field3: float; mut field4: float <- never}`

Mutable fields have two associated types, the type that can be *read* from the field, and the type that can be *written* to the field. If you specify a single type (e.g. `mut a: float`), it will be used as both the read and write type. 

In rare cases, you may want to specify separate read and write types, which can be done via `mut field: read_ty <- write_ty`. For example, in the type `{mut a: any <- float}`, only floats can be written to field `a`, but when the field is accessed, it will be read as type `any`.

* Tuple types: `int * str * float` is shorthand syntax for the record type `{_0: int; _1: str; _2: float}`.


* Variant types: ``[`Foo int | `Bar float]``

> Note: OCaml requires an "of" between the tag and type, e.g. ``[`Foo of int]``. In PolySubML, the "of" is optional.

> Note: The list of cases cannot be empty. `[]` is not a valid type. Use `never` instead.

* Inferred types: `_`

Each `_` in the source code creates a fresh inference variable whose type will be inferred by the compiler. It is useful if you only want to specify part of a type, e.g. `{a: int; b: _}`

* Recursive types: ``rec list = [`Some int * list | `None any]``

The general form of recursive types is `rec name = type`, where `name` can appear within `type`. In order to ensure that the recursive type is well-formed, `type` must be a record, function, variant, or recursive type.

Additionally, there is syntax for more advanced types that will be described later (polymorphic types, existential types, etc.)

### Polymorphism

Up until now, we've only dealt with *monomorphic* code, where each value has a single, specific type. However, in some cases, you may need *polymorphism* in order to facilitate code reuse.

For example, consider the identity function, `id` here:

```ocaml
let id = fun x -> x;

let _ = 1 + id 1;
let _ = 2.2 *. id -1.9;
```

`id` is a function that returns its argument unchanged and could theoretically work for any type of value. However, in monomorphic code, it has to operate on a single specific type. Therefore, if we try to use it on both `int`s and `float`s as shown above, we'll get a type error:

```
TypeError: Value is required to have type float here:
let _ = 1 + id 1;
let _ = 2.2 *. id -1.9;
            ^~~~~~~~~~  
However, that value may have type int originating here:
let id = fun x -> x;
let _ = 1 + id 1;
               ^  
let _ = 2.2 *. id -1.9;
Hint: To narrow down the cause of the type mismatch, consider adding an explicit type annotation here:
let id = fun x: _ -> x;
              +++       
let _ = 1 + id 1;
```

In order to make this work, we need to make `id` *generic* over its input type, so it can work for both ints and floats (as well as any other type).

#### Generic function definitions

To define a generic function, you need to add one or more *type parameters* after the `fun`, and then use those type parameters to annotate the type of the function, like so:

```ocaml
let id = fun (type t) (x: t): t -> x;

let _ = 1 + id 1;
let _ = 2.2 *. id -1.9;
```

Here, `t` is a type parameter for the function, representing a type that can arbitrarily be chosen by the caller on a per-call basis. Then we annotate the function argument and return type to show that it takes a value of type `t` and returns a value of type `t`. Whenever the function is called, `t` will be sustituted for the appropriate type, `int` or `float` in this example.

A function can also have multiple type parameters, in which case you just write `(type name1 name2...)`, e.g.

```ocaml
let swap = fun (type a b) (x: a, y: b): b * a -> (y, x);
print swap ("hello", false);
print swap ({x=42}, 7.8);
```

Type parameters are never inferred during type inference, so you need to specify every use of type parameters in the function's type signature explicitly. However, within the *body* of the function, the type parameters are replaced by ordinary abstract types which *can* be inferred, just like any other type.

```ocaml
fun (type t) (x: t): _ (* inferred any *) -> (
  let y: _ (* inferred t *) = x;
  y
)
```
In the above example, the first `_` is in the function signature, where `t` is a type parameter, and hence it cannot be inferred to `t`, and instead becomes `any`. However, the second `_` appears within the body of the function, where `t` is an ordinary type and can freely be inferred.

Note that parts of the function signature which do *not* reference its type parameters are still fully inferrable and hence can be omitted.


#### Generic function types

Generic function types can be written as `type parameters. function type`. For example, the `id` and `swap` functions above have the types `type t. t -> t` and `type a b. a * b -> b * a` respectively.


```ocaml
let id: type t. t->t = 
  fun (type t) (x: t): t -> x;

let swap: type a b. a * b -> b * a = 
  fun (type a b) (x: a, y: b): b * a -> (y, x);
```

Generic function types are just ordinary types like any other type, and hence can themselves be passed into functions, which is known as *higher rank types*. For example:

```ocaml
let f = fun (type t) (v: t, f: type u. t * u -> u): int * float -> (f (v, 1), f (v, 9.3));
```

Type parameter names are part of the function's type. This means that `type t. t -> t` and `type u. u -> u` are distinct types. However, order does not matter. `type a b. a * b -> b * a` and `type b a. a * b -> b * a` are considered the *same* type.

In order to avoid naming conflicts with nested polymorphic types, you can optionally rename type parameters with an *alias*, e.g. `(type a as b)`. In this case, the type parameter's *name* will still be `a` for the purpose of the type signature, but it is *referenced in the code* as `b`.

For example, consider the higher rank `f` function above, and suppose that we wanted both the inner and outer function to use the type parameter name `t`. We can achieve this by using an alias in the outer or inner function to avoid a name collision:

```ocaml
// Alias t to u in outer function
let f = fun (type t as u) (v: u, f: type t. u * t -> t): int * float -> (f (v, 1), f (v, 9.3));

// Alias t to u in inner function
let f = fun (type t) (v: t, f: type t as u. t * u -> u): int * float -> (f (v, 1), f (v, 9.3));
```

Aliasing can also be done within a type annotation. The following three type annotations are all equivalent:

```ocaml
(f: type t as u. u * (type t. u * t -> t) -> int * float);
(f: type t. t * (type t as u. t * u -> u) -> int * float);
(f: type t as foo. foo * (type t as bar. foo * bar -> bar) -> int * float);
```

#### Generic function instantiation

*Instantiation* is the process of replacing the type parameters in a generic function type with actual type values. Generic functions are implicitly instantiated whenever they are called.

You can also *explicitly* instantiate generic types using the syntax `expr[name1=type2; name2=type2; ...]`. For example, here we create a generic function `id` and then explicitly instantiate it with `t=int` and `t=float`:

```ocaml
let id: type t. t->t = 
  fun (type t) (x: t): t -> x;


let id_int: int -> int = id[t=int];
let id_float: float -> float = id[t=float];
```

Instantiation always replaces *all* of the function's type parameters at once. Any type parameters not explicitly specified are implicitly replaced with fresh inference variables. `f[t=int]` is equivalent to `f[t=int; a=_; b=_; c=_; ...]` for every possible parameter name.

This also means you can instantiate a function with inferred types for each parameter by just writing `f[]`. This is done implicitly at every call site - `f a ` is equivalent to `f[] a`. That is why `id 1`, `id "foo"`, etc. work even with no explicit instantiation of `id`.

However, **instantiation is not part of the subtyping order for polymorphic types**. Instantiation is done implicitly at every function call and also whenever requested explicitly via `[]`, but it is *not* done implcitly during type checking.

This means that `type t. t->t` is *not* a subtype of `int -> int` or `float -> float`. If you try to assign a generic function to a concrete type without first instantiating it, you'll get a type error. For example:


```ocaml
let id = fun (type t) (x: t): t -> x;
let id_int: int -> int = id;
```

results in a type error:

```
TypeError: Value is required to have type int here:
let id = fun (type t) (x: t): t -> x;
let id_int: int -> int = id;
                   ^~~       
However, that value may be a type parameter t originating here:
let id = fun (type t) (x: t): t -> x;
                              ^       
let id_int: int -> int = id;
Note: Type mismatch was detected starting from this expression:
let id = fun (type t) (x: t): t -> x;
let id_int: int -> int = id;
                         ^~  
```

The solution is to instantiate it explicitly with `[]` like this:

```ocaml
let id = fun (type t) (x: t): t -> x;
let id_int: int -> int = id[];
```

#### Partial instantiation and currying

In order to keep type checking fast, PolySubML requires that partially instantiated generics function types must be explicitly annotated before they can be further instantiated. Since `[]` instantiates *every* type parameter at once, you normally don't need to worry about this. However, it can happen when using nested generic types where the inner type depends on type parameters bound at multiple levels.

For example, suppose you have a curried generic function like this where each type parameter is chosen as late as possible:


```ocaml
let make_pair = fun (type a) (x: a): (type b. b -> a * b) ->
  fun (type b) (y: b): a * b -> 
    (x, y)
;

let pair = (make_pair 123) "bar";
```

If you try to call it normally like that, you'll get an error:

```
TypeError: Repeated instantiation of nested polymorphic type requires intervening type annotation:
;
let pair = ((make_pair 123): <type here>) "bar";
           +               ++++++++++++++        
Note: Type mismatch was detected starting from this expression:
;
let pair = (make_pair 123) "bar";
            ^~~~~~~~~         
```

The reason for this error is because the first call instantiates the type of `make_pair` with `a=int`, so that the type becomes `int -> (type b. b -> int * b)`. This means that the return type is `type b. b -> int * b`, which is a partially instantiated function (as it contains both the instantiated `a=int` and the non-instantiated `b` parameter). Since partially instantatied functions can't be further instantiated and function calls implicitly instantiate, this means it can't be called unless you add an explicit type annotation.

You can avoid this problem by defining all type parameters at the start for curried functions. As a bonus, this also makes the function signature much simpler:

```ocaml
let make_pair = fun (type a b) (x: a): (b -> a * b) ->
  fun y -> 
    (x, y)
;

let pair = (make_pair 123) "bar";
```

Note that in addition to avoiding the partial instantiation problem, defining the type parameters up front also means that you no longer need to repeat the function signature at every step in the chain. The inner function no longer requires any type annotations.

This approach does have the downside is that `b` is now fixed to a specific type as soon as you pass the first parameter. The partially applied function is no longer generic in its second parameter type. If you really need that, you're just going to have to bite the bullet and add extra type annotations.


#### You may not need polymorphism

If you are used to languages without subtyping, you may think you need polymorphism even in cases where it is unnecessary in PolySubML. For example, consider this function:

```ocaml
let none = fun (type t) _: [`None any | `Some t] -> `None {};
```

In a language without subtyping, this function would require polymorphism. However, in PolySubML, it actually doesn't require polymorphism at all! You can simplify the type like this:

```ocaml
let none = fun _: [`None any | `Some never] -> `None {};
```

More generally, **type parameters are only necessary if they appear in both covariant and contravariant position**. Roughly speaking, this means that type parameters are only necessary if they appear in both the function input *and* output. Type parameters which appear only in covariant position (i.e. output) can be replaced by `never` as in the example above. Type parameters which appear only in contravariant (i.e. input) position can be replaced by `any`.



#### Existential types

*Existential types* are the mirror image of generic types. A generic function has type parameters which can be substitued for any type by the caller. An existential type by contrast has type parameters representing *some* unknown type that can differ on a per-value basis.

In PolySubML, extential types are tied to records. You can deconstruct one by adding `type name;` to a record destructuring pattern, and then using that type parameter to annotate the other fields, like this:

```ocaml
let {type t; a: t; b: t->t} = {a=3; b=fun x->x+1};
print b b a; // 5
```

In this particular example, using an existential type is a bit pointless. The advantage of existential types is that it lets you work with many values with different types, as long as the individual type of each value matches a specific pattern. 

For example, we can change the code to take in a value that could contain ints or floats like this:

```ocaml
let r = if false then 
  {a=3; b=fun x->x+1}
else 
  {a=9.1; b=fun x->x*.2.0}
;

let {type t; a: t; b: t->t} = r;
print b b a; // 36.4
```

The actual, underlying type of `a` could be int or float. However, the use of an existential type means that the type is changed to an abstract type `t`, and likewise, `b` just has type `t->t`. This allows code to work with those values *without knowing anything about their actual types*.


An existential type can also have multiple type parameters. Unlike with generic functions, for existential types, each parameter is defined in a separate entry, e.g. `{type t; type u; type v;...}`:


```ocaml
let r = {a=3; f=fun x->(x, x+1); g=fun (x, y) -> x * y};

let {type t; type u; a: t; f: t->u; g: u->t} = r;
print g f a; // 12
```

#### Existential type instantiation

Existential types are implicitly instantiated whenever you define a record literal. Normally, the type parameters values are inferred, but you can also specify them explicitly if you want by adding `type t=...` to the record literal.

Here is the previous int/float example modified to explicitly specify the type parameter values:

```ocaml
let r = if false then 
  {type t=int; a=3; b=fun x->x+1}
else 
  {type t=float; a=9.1; b=fun x->x*.2.0}
;

let {type t; a: t; b: t->t} = r;
print b b a; // 36.4
```

Likewise, here's the second example modified with explicit types for `t` and `u`:

```ocaml
let r = {type t=int; type u=int * int; 
  a=3; f=fun x->(x, x+1); g=fun (x, y) -> x * y};

let {type t; type u; a: t; f: t->u; g: u->t} = r;
print g f a; // 12
```

> Note: The `type t=int;` entries in the record literal merely indicate *how to instantiate the existential type* that appears below it. They do not define a new type by themselves. This means that `let r = {type t=int; a: t; ...}` is not valid because `t` is not actually a type and hence can't be used to annotate the field `a`.


In addition to explicitly specifying the instantiation parameters on a record literal, you can also specify them on *any* expression using the `expr[[]]` syntax. This is similar to the explicit generic instantiation syntax in the previous section, but uses two brackets instead of one. 

For example, the above example can be modified to specify the instantiation parameters later on like this:


```ocaml
let r = {a=3; f=fun x->(x, x+1); g=fun (x, y) -> x * y};

let r = r[[t=int; u=int * int]];

let {type t; type u; a: t; f: t->u; g: u->t} = r;
print g f a; // 12
```

Existential *type* syntax also uses `{type t; type u; ...}`, just like existential record patterns:

```ocaml
let r = {a=3; f=fun x->(x, x+1); g=fun (x, y) -> x * y};

// Annotate with an explicit existential type
let r = (r: {type t; type u; a: t; f: t->u; g: u->t});

let {type t; type u; a: t; f: t->u; g: u->t} = r;
print g f a; // 12
```


Existential types follow the same rules as generic types described previously. Uses of type parameters in the destructuring pattern *must* be explicitly annotated, but in the code after that, they become ordinary abstract types which are freely inferrable. 

Like generic types, existential types can be freely nested and type parameters can be aliased. Partially instantiated existential types cannot be further instantiated without an explicit type annotation.

The `type` entries in existential record literals, record types, and record patterns *must* come before any fields. `{a=3; type t=int}` is not a legal expression, `{a: t; type t}` is not a legal type, etc.

#### Type unions and intersections

For ordinary monomorphic types, type unions (`t | u`) and intersections (`t & u`) can always be simplified down to a single type. For example, you can simplify the union of two types using the following rules:

* `t | never = t`
* `t | any = any`
* `t | t = t`
* `t | u = any` when `t` and `u` *have different type constructors* (e.g. one is a record and one is a function, or one is a variant and one is an int, etc.)
* `(a -> b) | (c -> d) = (a & c) -> (b | d)`
* `{a: t1; b: t2} | {b: t3; c: t4} = {b: t3 | t4}`
* ``[`A t1 | `B t2] | [`B t3 | `C t4] = [`A t1 | `B (t2 | t3) | `C t4]``

And likewise, intersection types can be simplified using a flipped version of the same rules.

However, for *polymorphic* types, it is not possible to fully simplify unions and intersections in this way. For example, if you want to take the union of `type t. t -> t * int` and `type t. t -> str * t`, you can simplify it down to `type t. t -> (t | str) * (t | int)`, but there is no way to further simplify the `t | str` or `t | int` parts.

Therefore, PolySubML supports a limited form of explicit type unions and intersections in order to make these types representable.

A type is called *union-eligible* if it is 
1) A type parameter of a generic function *type* (note: *not* a function *definition*) that appears in covariant position or
2) A type parameter of an existential *type* (note: *not* an existential *pattern*) that appears in contravariant position.

All other types are called *union-ineligible*.

A union can contain any number of union-eligible types and *at most one* union-ineligible type. For example, `type t. t -> (t | str) * (t | int)` is a valid type because `t | str` and `t | int` both contain only one union-ineligible type (`int` and `str` respectively).

Likewise, a type is *intersection-eligible* if it is 
1) A type parameter of a generic function *type* that appears in *contravariant* position or
2) A type parameter of an existential *type* that appears in *covariant* position.

All other types are called *intersection-ineligible*. An intersection can contain any number of intersection-eligible types and at most one intersection-ineligible type.

Additionally in order to avoid programmer confusion, `any` and `never` cannot appear in unions and intersections, since these can always be simplified away.

The above rules may sound arbitrary, but they are a) just powerful enough to make every possible type representable while also b) being just restrictive enough to allow for fast type checking. For the most part, you will never have to worry about them, and they are included here just for completeness.

### Loops and recursion

When working with recursive data structures, functional programming style relying on recursion can be convenient. However, the downside of recursion is that programming language runtimes tend to have strict limit on recursion depth, making recursive functions crash when working with large data, even when the equivalent imperative style code would have no problems.

For example, we can define a function to construct a linked list of ints in a range like this:

```ocaml
// Create a linked list of ints from s..e
let rec make_list = fun {s; e} -> 
    if s >= e then 
    `None {}
    else   
    `Some {h=s; t=make_list {s=s+1; e}}
;

print make_list {s=1; e=100};
```

This works fine for small values, but will crash when using sufficiently deep recursion. In the case of the PolySubML web demo, it crashes around `e=16000`:

```
>> make_list {s=1; e=16000}
An error occurred during evaluation in the repl: InternalError: too much recursion
```

Note that PolySubML does not currently implement tail call optimization, which means that even tail-recursive functions are still subject to recursion limits. 

Fortunately, PolySubML also supports imperative style iteration via the `loop` expression. This takes the form `loop body`, where `body` is an expression that must evaluate to a value of type ``[`Break t | `Continue any]`` for some type `t`. The body expression is evaluated repeatedly until it returns `Break`, and the wrapped value becomes the value of the loop expression as a whole.

For example, ``loop `Break 42`` is equivalent to just `42`, while ``loop `Continue 0`` is an infinite loop. In order to do something more interesting, you need to use mutable state, which means a record with mutable fields. Here's an imperative FizzBuzz implementation using a loop and mutable counter:

```ocaml
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
```



