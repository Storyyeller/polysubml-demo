// Test 51: Mutable fields with variant types (fixed)
let state_machine = (
    let state = {mut current = `Idle {}};
    let transition = fun new_state -> state.current <- new_state;
    {state; transition}
);

print state_machine.state.current;

// Transition to different states
state_machine.transition `Running {speed=10};
print state_machine.state.current;

state_machine.transition `Stopped {reason="user requested"};
print state_machine.state.current;

// Test with pattern matching
match state_machine.state.current with
| `Idle _ -> print "idle"
| `Running {speed} -> print "running at", speed
| `Stopped {reason} -> print "stopped:", reason;