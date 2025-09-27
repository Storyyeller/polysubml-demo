// Test 51: Mutable fields with variant types
let state_machine = {
    mut current = `Idle {};
    transition = fun new_state -> current <- new_state;
};

print state_machine.current;

// Transition to different states
state_machine.transition `Running {speed=10};
print state_machine.current;

state_machine.transition `Stopped {reason="user requested"};
print state_machine.current;

// Test with pattern matching
match state_machine.current with
| `Idle _ -> print "idle"
| `Running {speed} -> print "running at", speed
| `Stopped {reason} -> print "stopped:", reason;