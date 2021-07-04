#![cfg(feature = "test-bpf")]

mod helpers;

use rndr::state::AccountType;
use {
    helpers::*,
    rndr::{instruction::fund_job, processor::process_instruction},
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
    let test_source_token = TestToken::add(&mut test, test_mint.pubkey, AMOUNT);
    let test_escrow = TestEscrow::add(&mut test, test_mint.pubkey, ZERO);
    let authority = test_source_token.owner.pubkey();

    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    let source_token_balance_before =
        get_token_balance(&mut banks_client, test_source_token.pubkey).await;
    let escrow_balance_before =
        get_token_balance(&mut banks_client, test_escrow.associated_token).await;

    assert_eq!(source_token_balance_before, AMOUNT);
    assert_eq!(escrow_balance_before, ZERO);

    let escrow = get_escrow(&mut banks_client, test_escrow.pubkey).await;
    assert_eq!(escrow.amount, ZERO);

    let mut transaction = Transaction::new_with_payer(
        &[fund_job(
            rndr::id(),
            AMOUNT,
            test_mint.pubkey,
            payer.pubkey(),
            test_source_token.pubkey,
            authority,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &test_source_token.owner], recent_blockhash);

    assert!(banks_client.process_transaction(transaction).await.is_ok());

    let source_token_balance_after =
        get_token_balance(&mut banks_client, test_source_token.pubkey).await;
    let escrow_balance_after =
        get_token_balance(&mut banks_client, test_escrow.associated_token).await;

    assert_eq!(source_token_balance_after, ZERO);
    assert_eq!(escrow_balance_after, AMOUNT);

    let escrow = get_escrow(&mut banks_client, test_escrow.pubkey).await;
    assert_eq!(escrow.amount, AMOUNT);

    let (job_pubkey, _bump_seed) = Pubkey::find_program_address(
        &[b"job", test_escrow.pubkey.as_ref(), authority.as_ref()],
        &rndr::id(),
    );
    let job = get_job(&mut banks_client, job_pubkey).await;

    assert_eq!(job.account_type, AccountType::JobV1);
    assert_eq!(job.authority, authority);
    assert_eq!(job.amount, AMOUNT);
}
