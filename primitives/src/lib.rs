#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

pub type Address = [u8; 20];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub balance: u64,
}

// The accounts that will be touched
pub type StateWitness = BTreeMap<Address, Account>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub amount: u64,
}

// Ethereum's ExecutionPayload concept (the block data)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPayload {
    pub txs: Vec<Transaction>,
}

// The total package handed from the Host to the Guest (zkVM)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuestInput {
    pub state_witness: StateWitness,
    pub payload: ExecutionPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionError {
    SenderNotFound,
    ReceiverNotFound,
    InsufficientBalance,
}

impl GuestInput {
    /// Ensure the witness is structurally correct
    pub fn validate(&self) -> Result<(), ExecutionError> {
        for tx in &self.payload.txs {
            // Validate sender exists
            if !self.state_witness.contains_key(&tx.from) {
                return Err(ExecutionError::SenderNotFound);
            }
            // Validate receiver exists
            if !self.state_witness.contains_key(&tx.to) {
                return Err(ExecutionError::ReceiverNotFound);
            }
        }
        Ok(())
    }

    /// The State Transition Function
    /// Takes the pre-state and applies the transactions.
    pub fn replay(&self) -> Result<StateWitness, ExecutionError> {
        // Create a mutable working state from the pre-state witness
        let mut working_state = self.state_witness.clone();

        // Execute the payload
        for tx in &self.payload.txs {
            // validate() guaranteed sender exists
            let sender = working_state.get_mut(&tx.from).unwrap();

            // Check sufficient balance
            if sender.balance < tx.amount {
                return Err(ExecutionError::InsufficientBalance);
            }

            // Deduct from sender
            sender.balance -= tx.amount;

            // validate() guaranteed receiver exists
            let receiver = working_state.get_mut(&tx.to).unwrap();

            // Add to receiver safely (preventing overflow panics)
            receiver.balance = receiver
                .balance
                .checked_add(tx.amount)
                .expect("Balance overflow");
        }

        Ok(working_state)
    }

    /// Orchestrates validation and replay
    /// Returns the newly computed post-state
    pub fn execute(&self) -> Result<StateWitness, ExecutionError> {
        // Validate structural integrity
        self.validate()?;
        // Replay the execution to get the resulting state
        self.replay()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    // Helper to generate a mock address
    fn mock_addr(id: u8) -> Address {
        let mut addr = [0u8; 20];
        addr[19] = id;
        addr
    }

    // Helper to set up a valid base input.
    // Returns the input and the expected outcome
    fn setup_valid_input() -> (GuestInput, StateWitness) {
        let alice = mock_addr(1);
        let bob = mock_addr(2);

        let mut state_witness = BTreeMap::new();
        state_witness.insert(alice, Account { balance: 100 });
        state_witness.insert(bob, Account { balance: 50 });

        let tx = Transaction {
            from: alice,
            to: bob,
            amount: 10,
        };

        let mut expected_post_state = BTreeMap::new();
        expected_post_state.insert(alice, Account { balance: 90 });
        expected_post_state.insert(bob, Account { balance: 60 });

        let input = GuestInput {
            state_witness,
            payload: ExecutionPayload { txs: vec![tx] },
        };

        (input, expected_post_state)
    }

    #[test]
    fn test_valid_execution() {
        let (input, expected_state) = setup_valid_input();
        // Asserts that the STF successfully computed the expected state
        assert_eq!(input.execute(), Ok(expected_state));
    }

    #[test]
    fn test_fail_missing_receiver() {
        let (mut input, _) = setup_valid_input();
        // Change tx destination to an unknown address
        input.payload.txs[0].to = mock_addr(99);

        assert_eq!(input.execute(), Err(ExecutionError::ReceiverNotFound));
    }

    #[test]
    fn test_fail_insufficient_balance() {
        let (mut input, _) = setup_valid_input();
        // Alice only has 100, try to send 200
        input.payload.txs[0].amount = 200;

        assert_eq!(input.execute(), Err(ExecutionError::InsufficientBalance));
    }

    #[test]
    fn test_fail_tampered_witness_missing_sender() {
        let (mut input, _) = setup_valid_input();
        let alice = mock_addr(1);
        // Remove sender from the witness
        input.state_witness.remove(&alice);

        assert_eq!(input.execute(), Err(ExecutionError::SenderNotFound));
    }
}
