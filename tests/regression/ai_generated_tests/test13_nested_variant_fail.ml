// Test 13: Nested variant pattern that should fail
let x = `Foo `NotBar 0;

match x with
| `Foo `Bar _ -> "hello"
| _ -> "world";