#![cfg(feature = "test-bpf")]

mod helpers;

use {
    helpers::*,
    rndr::{instruction::disburse_funds, processor::process_instruction},
    solana_program_test::*,
    solana_sdk::{pubkey::Pubkey, signature::Signer, transaction::Transaction},
};

#[tokio::test]
async fn test_success() {
    let mut test = ProgramTest::new("rndr", rndr::id(), processor!(process_instruction));

    const ZERO: u64 = 0;
    const DECIMALS: u64 = 1_000_000_000;
    const AMOUNT: u64 = 1 * DECIMALS;

    let test_mint = TestMint::add(&mut test, 9, 100 * DECIMALS);
    let test_escrow = TestEscrow::add(&mut test, test_mint.pubkey, AMOUNT);
    let test_destination_token = TestToken::add(&mut test, test_mint.pubkey, ZERO);
    let test_job = TestJob::add(
        &mut test,
        test_escrow.pubkey,
        test_destination_token.owner.pubkey(),
        AMOUNT,
    );

    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    let escrow_balance_before =
        get_token_balance(&mut banks_client, test_escrow.associated_token).await;
    let destination_token_balance_before =
        get_token_balance(&mut banks_client, test_destination_token.pubkey).await;

    assert_eq!(escrow_balance_before, AMOUNT);
    assert_eq!(destination_token_balance_before, ZERO);

    let escrow = get_escrow(&mut banks_client, test_escrow.pubkey).await;
    assert_eq!(escrow.amount, AMOUNT);

    let job = get_job(&mut banks_client, test_job.pubkey).await;
    assert_eq!(job.amount, AMOUNT);

    let mut transaction = Transaction::new_with_payer(
        &[disburse_funds(
            rndr::id(),
            AMOUNT,
            test_mint.pubkey,
            test_destination_token.pubkey,
            test_job.pubkey,
            test_escrow.owner.pubkey(),
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &test_escrow.owner], recent_blockhash);

    assert!(banks_client.process_transaction(transaction).await.is_ok());

    let escrow_balance_after =
        get_token_balance(&mut banks_client, test_escrow.associated_token).await;
    let destination_token_balance_after =
        get_token_balance(&mut banks_client, test_destination_token.pubkey).await;

    assert_eq!(escrow_balance_after, ZERO);
    assert_eq!(destination_token_balance_after, AMOUNT);

    let escrow = get_escrow(&mut banks_client, test_escrow.pubkey).await;
    assert_eq!(escrow.amount, ZERO);

    let job = get_job(&mut banks_client, test_job.pubkey).await;
    assert_eq!(job.amount, ZERO);
}
