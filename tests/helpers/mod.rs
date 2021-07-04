#![allow(dead_code)]

use {
    rndr::state::{Escrow, InitEscrowParams},
    solana_program::{program_option::COption, program_pack::Pack, pubkey::Pubkey},
    solana_program_test::*,
    solana_sdk::{
        account::Account,
        signature::{Keypair, Signer},
    },
    spl_token::state::{Account as Token, AccountState, Mint},
};

trait AddPacked {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    );
}

impl AddPacked for ProgramTest {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    ) {
        let mut account = Account::new(amount, T::get_packed_len(), owner);
        data.pack_into_slice(&mut account.data);
        self.add_account(pubkey, account);
    }
}

pub struct TestMint {
    pub pubkey: Pubkey,
    pub authority: Keypair,
    pub decimals: u8,
}

pub fn add_token_mint(test: &mut ProgramTest, decimals: u8, supply: u64) -> TestMint {
    let pubkey = Pubkey::new_unique();
    let authority = Keypair::new();

    test.add_packable_account(
        pubkey,
        u32::MAX as u64,
        &Mint {
            is_initialized: true,
            decimals,
            mint_authority: COption::Some(authority.pubkey()),
            supply,
            ..Mint::default()
        },
        &spl_token::id(),
    );

    TestMint {
        pubkey,
        authority,
        decimals,
    }
}

pub struct TestAccount {
    pub pubkey: Pubkey,
    pub mint: Pubkey,
    pub owner: Keypair,
}

pub fn add_token_account(test: &mut ProgramTest, mint: Pubkey, amount: u64) -> TestAccount {
    let pubkey = Pubkey::new_unique();
    let owner = Keypair::new();

    test.add_packable_account(
        pubkey,
        u32::MAX as u64,
        &Token {
            mint,
            owner: owner.pubkey(),
            amount,
            state: AccountState::Initialized,
            is_native: COption::None,
            ..Token::default()
        },
        &spl_token::id(),
    );

    TestAccount {
        pubkey,
        mint,
        owner,
    }
}

pub struct TestEscrow {
    pub pubkey: Pubkey,
    pub owner: Keypair,
}

pub fn add_escrow(test: &mut ProgramTest, token_account: Pubkey) -> TestEscrow {
    let (pubkey, _bump_seed) = Pubkey::find_program_address(
        &[b"escrow", token_account.as_ref(), spl_token::id().as_ref()],
        &rndr::id(),
    );

    let owner = Keypair::new();

    test.add_packable_account(
        pubkey,
        u32::MAX as u64,
        &Escrow::new(InitEscrowParams {
            owner: owner.pubkey(),
        }),
        &rndr::id(),
    );

    TestEscrow { pubkey, owner }
}
