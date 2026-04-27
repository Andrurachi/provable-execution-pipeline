# Provable Execution: A Lean Ethereum Pipeline
[See on HackMD](https://hackmd.io/@Andrurachi/BytTTGaaZe)

This project was developed as an exploration of Lean Ethereum L1 statelessness, zkVM proving architectures, and the cryptographic boundaries between Provers and Verifying Nodes. It is intended to build a ground-up understanding of the infrastructure required to generate validity proofs for Ethereum block execution.

### Project Components
* [State Transition Primitives:](https://hackmd.io/@Andrurachi/Skb70zTTZx) A `no_std` library defining simplified Ethereum-like accounts, transactions, and the deterministic State Transition Function (STF).
* [The Guest zkVM:](https://hackmd.io/@Andrurachi/HkMS0MaTZx) The constrained RISC-V runtime environment that blindly executes the STF and commits the computed state to the public journal.
* [The Host Node:](https://hackmd.io/@Andrurachi/Sy9L0faTWl) The asynchronous Rust application that orchestrates SP1 proof generation and also acts as the L1 stateless verifier.

---

### Phase 1: The Core STF (`primitives`)
The first phase of the project focused on defining the data structures and the execution logic independently from the proving system. 

A custom `GuestInput` struct acts as the bridge, packaging the Pre-State Witness and the Execution Payload (the block's transactions). The `execute()` method orchestrates structural validation and state replay, returning the newly computed `StateWitness`.

*Key Learnings:*
* Designing `no_std` compatible state logic using `alloc::collections::BTreeMap`.
* Understanding the L1 stateless architectural shift: The STF strictly computes the new state rather than comparing it against an expected target inside the VM.

### Phase 2: The Proving Engine (`guest` and `host`)
The second phase transitioned from pure logic to zero-knowledge execution using SP1 v6.

The `guest` program acts as a pure calculator: it reads the serialized bytes via `sp1_zkvm::io::read`, executes the STF, and pushes the resulting state into the public `journal`. The `host` leverages `tokio` to manage the asynchronous `ProverClient`, taking the compiled RISC-V ELF and generating the interactive STARK proof.

*Key Learnings:*
* Serializing complex Rust structs into the zkVM via `SP1Stdin`.
* Understanding the role of the Program Image ID (Verifying Key) as the cryptographic fingerprint of the STF.

### Phase 3: The L1 Verification
With the proof generated, the final phase explored how a Ethereum node would validate the block without re-executing the transactions.

The Host also acts as the verifying node. It holds the block proposer's claimed `expected_post_state`. To optimize for computational limits, the node implements a Fast-Fail Optimization: it reads the unverified state from the Proof's Journal and compares it against the expected state before spending CPU cycles on the heavy cryptographic verification.

*The Pipeline:*
1.  **Extract:** Read the `StateWitness` from the `public_values` journal.
2.  **Fast-Fail:** Assert `journal_state == expected_post_state`. If false, drop the block immediately.
3.  **Cryptographic Verification:** Mathematically validate that the `proof` matches the `vk` and binds to the provided journal.

---

### Possible Next Iteration Plan: Production Integration
While this project demonstrates the L1 execution architecture using a Core STARK, it relies on mocked data. A next goal for this project is to integrate real Ethereum tooling to prepare for production environments. This involves:

1.  **The Reth Bridge:** Replacing custom arrays with standard `alloy` types and fetching real JSON-RPC execution payloads.
2.  **Witness Validation Engine:** Building pre-flight checks to deterministically guarantee a witness is complete before it enters the zkVM, preventing expensive panics.
3.  **Ethereum Alignment (KZG Plonk):** Activating the SNARK wrapper pipeline ([check code here](https://github.com/Andrurachi/provable-execution-pipeline/blob/ecc9994e60d9e235893d28466f5e865a7dd0028d/host/src/main.rs#L62-L70)) to compress the STARK into a cheap-to-verify KZG proof using the EVM's `BN254` precompile.
