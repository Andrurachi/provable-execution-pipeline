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