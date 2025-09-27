// Test 41: Simple higher-rank type test
// Test if we can even assign a polymorphic function to higher-rank type
let id = fun (type t) (x: t): t -> x;
let higher_rank_id: type t. t -> t = id[];