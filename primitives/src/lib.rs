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