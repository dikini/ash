#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz the effect lattice operations
    if data.len() < 2 {
        return;
    }
    
    use ash_core::effect::Effect;
    
    // Parse effects from fuzz input
    let e1 = match data[0] % 4 {
        0 => Effect::Epistemic,
        1 => Effect::Deliberative,
        2 => Effect::Evaluative,
        _ => Effect::Operational,
    };
    
    let e2 = match data[1] % 4 {
        0 => Effect::Epistemic,
        1 => Effect::Deliberative,
        2 => Effect::Evaluative,
        _ => Effect::Operational,
    };
    
    // Test lattice properties
    let join = e1.join(e2);
    let meet = e1.meet(e2);
    
    // Idempotence: a ⊔ a = a, a ⊓ a = a
    assert_eq!(e1.join(e1), e1);
    assert_eq!(e1.meet(e1), e1);
    
    // Commutativity: a ⊔ b = b ⊔ a, a ⊓ b = b ⊓ a
    assert_eq!(join, e2.join(e1));
    assert_eq!(meet, e2.meet(e1));
    
    // Absorption: a ⊔ (a ⊓ b) = a, a ⊓ (a ⊔ b) = a
    assert_eq!(e1.join(e1.meet(e2)), e1);
    assert_eq!(e1.meet(e1.join(e2)), e1);
});
