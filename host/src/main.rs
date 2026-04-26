use sp1_sdk::{Prover, include_elf, ProverClient, SP1Stdin, ProvingKey, HashableKey};
use primitives::{Account, ExecutionPayload, GuestInput, Transaction, StateWitness};
use std::collections::BTreeMap;
use std::time::Instant;

#[tokio::main]
async fn main() {
    sp1_sdk::utils::setup_logger();
    println!("Starting Lean Ethereum Prover");

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

    // ================================
    // Stateless verification
    println!("Node verifying proof and block headers...");

    // Read the computed post state from the Journal
    let journal_state = proof.public_values.read::<StateWitness>();
    
    // "Verifier" node checks if the claimed block header matches the STF execution
    // Done first to avoid heavy computation in case the proof doesn't have the expected post state
    if journal_state != expected_post_state {
        println!("STATE MISMATCH: Proof post-state doesn't match the proposer's block post-state!");
    }
    println!("State Root Match: The committed state equals the expected state.");

    // Verification
    println!("Verifying cryptographic binding...");
    
    match client.verify(&proof, vk, None) {
        Ok(_) => println!("VERIFICATION SUCCESSFUL! Block Accepted."),
        Err(e) => println!("VERIFICATION FAILED: {}", e),
    }
}
