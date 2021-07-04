#![deny(missing_docs)]

//! A RNDR program for the Solana blockchain.

pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

solana_program::declare_id!("7g4aX3DYhk6xHiGuoAbEnVTp9HMgLqyENoK53AVm267E");
