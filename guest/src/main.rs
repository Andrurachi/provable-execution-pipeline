// main fn will be called by sp1, not an operating system
#![no_main]

// Defines the entry point of the zkVM  
sp1_zkvm::entrypoint!(main);

use primitives::GuestInput;

pub fn main() {
    // Read the input from the host environment (SP1 deserializes the bytes sent by the host into the GuestInput struct)
    let input = sp1_zkvm::io::read::<GuestInput>();

    // Execute the STF and get the computed post state
    let computed_post_state = input.execute().expect("STF Execution Failed");

    // Commit the computed_post_state to the Journal
    sp1_zkvm::io::commit(&computed_post_state);
}