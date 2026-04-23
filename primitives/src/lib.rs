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
    pub expected_post_state: StateWitness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionError {
    SenderNotFound,
    ReceiverNotFound,
    InsufficientBalance,
    PostStateMismatch,
}

impl GuestInput {
    /// The entry point for the zkVM guest to verify the execution.
    pub fn verify(&self) -> Result<(), ExecutionError> {
        // Create a mutable working state from the pre-state witness
        let mut working_state = self.state_witness.clone();

        // Execute the payload
        for tx in &self.payload.txs {
            // Validate sender exists and get mutable reference
            let sender_account = working_state
                .get_mut(&tx.from)
                .ok_or(ExecutionError::SenderNotFound)?;

            // Check sufficient balance
            if sender_account.balance < tx.amount {
                return Err(ExecutionError::InsufficientBalance);
            }

            // Deduct from sender
            sender_account.balance -= tx.amount;

            // Validate receiver exists
            let receiver_account = working_state
                .get_mut(&tx.to)
                .ok_or(ExecutionError::ReceiverNotFound)?;

            // Add to receiver safely (preventing overflow panics)
            receiver_account.balance = receiver_account.balance
                .checked_add(tx.amount)
                .expect("Balance overflow"); 
        }

        // Verify the final working state matches the claimed expected_post_state
        if working_state != self.expected_post_state {
            return Err(ExecutionError::PostStateMismatch);
        }

        Ok(())
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

    // Helper to set up a valid base input
    fn setup_valid_input() -> GuestInput {
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

        GuestInput {
            state_witness,
            payload: ExecutionPayload { txs: vec![tx] },
            expected_post_state,
        }
    }

    #[test]
    fn test_valid_execution() {
        let input = setup_valid_input();
        assert_eq!(input.verify(), Ok(()));
    }

    #[test]
    fn test_fail_missing_receiver() {
        let mut input = setup_valid_input();
        // Change tx destination to an unknown address
        input.payload.txs[0].to = mock_addr(99); 
        
        assert_eq!(input.verify(), Err(ExecutionError::ReceiverNotFound));
    }

    #[test]
    fn test_fail_insufficient_balance() {
        let mut input = setup_valid_input();
        // Alice only has 100, try to send 200
        input.payload.txs[0].amount = 200; 
        
        assert_eq!(input.verify(), Err(ExecutionError::InsufficientBalance));
    }

    #[test]
    fn test_fail_incorrect_post_state() {
        let mut input = setup_valid_input();
        // Tamper with the expected post state (attacker claims Bob gets 100)
        let bob = mock_addr(2);
        input.expected_post_state.get_mut(&bob).unwrap().balance = 100;
        
        assert_eq!(input.verify(), Err(ExecutionError::PostStateMismatch));
    }

    #[test]
    fn test_fail_tampered_witness_missing_sender() {
        let mut input = setup_valid_input();
        let alice = mock_addr(1);
        // Remove sender from the witness
        input.state_witness.remove(&alice); 
        
        assert_eq!(input.verify(), Err(ExecutionError::SenderNotFound));
    }
}