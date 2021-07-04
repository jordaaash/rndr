#![cfg(feature = "test-bpf")]

mod helpers;

use {
    helpers::*,
    rndr::{instruction::set_escrow_owner, processor::process_instruction},
    solana_program_test::*,
    solana_sdk::{pubkey::Pubkey, signature::Signer, transaction::Transaction},
};

#[tokio::test]
async fn test_success() {
    let mut test = ProgramTest::new("rndr", rndr::id(), processor!(process_instruction));

    const ZERO: u64 = 0;
    const DECIMALS: u64 = 1_000_000_000;

    let test_mint = TestMint::add(&mut test, 9, 100 * DECIMALS);
    let test_escrow = TestEscrow::add(&mut test, test_mint.pubkey, ZERO);
    let new_owner = Pubkey::new_unique();

    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    let escrow = get_escrow(&mut banks_client, test_escrow.pubkey).await;
    assert_eq!(escrow.owner, test_escrow.owner.pubkey());

    let mut transaction = Transaction::new_with_payer(
        &[set_escrow_owner(
            rndr::id(),
            test_escrow.pubkey,
            test_escrow.owner.pubkey(),
            new_owner,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &test_escrow.owner], recent_blockhash);

    assert!(banks_client.process_transaction(transaction).await.is_ok());

    let escrow = get_escrow(&mut banks_client, test_escrow.pubkey).await;
    assert_eq!(escrow.owner, new_owner);
}
