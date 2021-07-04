#![cfg(feature = "test-bpf")]

mod helpers;

use rndr::state::AccountType;
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

    const ZERO: u64 = 0;
    const DECIMALS: u64 = 1_000_000_000;

    let test_mint = TestMint::add(&mut test, 9, 100 * DECIMALS);
    let owner = Keypair::new();

    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[init_escrow(
            rndr::id(),
            owner.pubkey(),
            test_mint.pubkey,
            payer.pubkey(),
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    assert!(banks_client.process_transaction(transaction).await.is_ok());

    let (escrow_pubkey, _bump_seed) = Pubkey::find_program_address(
        &[
            b"escrow",
            test_mint.pubkey.as_ref(),
            spl_token::id().as_ref(),
        ],
        &rndr::id(),
    );
    let escrow = get_escrow(&mut banks_client, escrow_pubkey).await;

    assert_eq!(escrow.account_type, AccountType::EscrowV1);
    assert_eq!(escrow.owner, owner.pubkey());
    assert_eq!(escrow.amount, ZERO);
}
