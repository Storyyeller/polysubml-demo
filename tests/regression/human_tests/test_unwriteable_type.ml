let _ = fun x -> (
    let a: rec f=(type T. any -> T -> f) = x;
    let b: any -> rec f=(type T. any -> T -> f) = x;

    let c: _ (*what type goes here?*) = if x then a else b;

    let _: rec f=(type T. never -> T -> f) = c;
    let _: never -> rec f=(type T. never -> T -> f) = c;

    0
);