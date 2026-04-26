use sp1_sdk::{Prover, include_elf, ProverClient, SP1Stdin, ProvingKey, HashableKey};
use primitives::{Account, ExecutionPayload, GuestInput, Transaction};
use std::collections::BTreeMap;
use std::time::Instant;

#[tokio::main]
async fn main() {
    sp1_sdk::utils::setup_logger();
    println!("Starting Lean Ethereum Prover POC");

    // Construct the mock block
    println!("Constructing mock block and witness...");
    
    let mut alice = [0u8; 20]; alice[19] = 1;
    let mut bob = [0u8; 20]; bob[19] = 2;

    let mut state_witness = BTreeMap::new();
    state_witness.insert(alice, Account { balance: 100 });
    state_witness.insert(bob, Account { balance: 50 });

    let tx = Transaction { from: alice, to: bob, amount: 10 };

    // Block proposed for that slot
    let mut expected_post_state = BTreeMap::new();
    expected_post_state.insert(alice, Account { balance: 90 });
    expected_post_state.insert(bob, Account { balance: 60 });

    let input_data = GuestInput {
        state_witness,
        payload: ExecutionPayload { txs: vec![tx] },
        expected_post_state,
    };

    // The async Prover and ELF
    let guest_elf = include_elf!("guest");

    // Initialize the asynchronous Prover Client
    let client = ProverClient::from_env().await;

    // The setup phase creates the Image ID (Program Hash)
    let pk = client.setup(guest_elf).await.expect("Failed to setup keys");
    let vk = pk.verifying_key();
    println!("Program Image ID: 0x{}", hex::encode(vk.bytes32()));

    let mut stdin = SP1Stdin::new();
    stdin.write(&input_data);

    // Execution and Proving
    println!("Generating core STARK proof...");
    let start_time = Instant::now();
    
    let mut proof = client
        .prove(&pk, stdin)
        .await
        .expect("Failed to generate proof");

    println!("Proof generated in {:?}", start_time.elapsed());

    // Read the Journal
    let journal_result = proof.public_values.read::<bool>();
    println!("Journal output (Did execution succeed?): {}", journal_result);
    
    if !journal_result {
        println!("State transition is invalid.");
    }

    // Verification
    println!("Verifying cryptographic binding...");
    
    match client.verify(&proof, vk, None) {
        Ok(_) => println!("VERIFICATION SUCCESSFUL! The math checks out."),
        Err(e) => println!("VERIFICATION FAILED: {}", e),
    }
}
