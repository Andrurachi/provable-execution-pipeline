// main fn will be called by sp1, not an operating system
#![no_main]

// Defines the entry point of the zkVM  
sp1_zkvm::entrypoint!(main);

use primitives::GuestInput;

pub fn main() {
    // Read the input from the host environment (SP1 deserializes the bytes sent by the host into the GuestInput struct)
    let input = sp1_zkvm::io::read::<GuestInput>();

    // Run the Verification (Validation + Replay)
    let result = input.verify();

    // Handle the outcome and Commit Public Values
    match result {
        Ok(_) => {
            // A verifier will read this public value to know the execution was valid.
            sp1_zkvm::io::commit(&true);
        }
        Err(_err) => {
            sp1_zkvm::io::commit(&false);
            
            // If the state transition is invalid, the program should panic, so no proof should be generated. 
            // But for a POC testing failure modes, committing false is fine :)
        }
    }
}