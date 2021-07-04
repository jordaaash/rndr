#![cfg(feature = "test-bpf")]

mod helpers;

use {
    helpers::*,
    rndr::{instruction::init_escrow, processor::process_instruction},
    solana_program_test::*,
    solana_sdk::{
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::Transaction,
    },
};

#[tokio::test]
async fn test_success() {
    let mut test = ProgramTest::new("rndr", rndr::id(), processor!(process_instruction));

    let token_mint = add_token_mint(&mut test, 9, 100_000_000_000);
    let owner = Keypair::new();

    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[init_escrow(
            rndr::id(),
            owner.pubkey(),
            payer.pubkey(),
            token_mint.pubkey,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    assert!(banks_client.process_transaction(transaction).await.is_ok());
}
