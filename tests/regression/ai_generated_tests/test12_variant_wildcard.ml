// Test 12: Wildcard matches and order independence
let handle_shape = fun shape ->
    match shape with
    | v -> "unknown shape"
    | `Circle _ -> "circle"
    | `Square _ -> "square";

print handle_shape `Circle {r=1.0};
print handle_shape `Triangle {a=1.0; b=2.0; c=3.0};