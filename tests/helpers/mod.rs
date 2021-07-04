#![allow(dead_code)]

use rndr::state::{InitJobParams, Job};
use spl_associated_token_account::get_associated_token_address;
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

pub async fn get_account(banks_client: &mut BanksClient, pubkey: Pubkey) -> Account {
    banks_client.get_account(pubkey).await.unwrap().unwrap()
}

pub async fn get_mint(banks_client: &mut BanksClient, pubkey: Pubkey) -> Mint {
    let account = get_account(banks_client, pubkey).await;
    Mint::unpack(&account.data).unwrap()
}

pub async fn get_token(banks_client: &mut BanksClient, pubkey: Pubkey) -> Token {
    let account = get_account(banks_client, pubkey).await;
    Token::unpack(&account.data).unwrap()
}

pub async fn get_token_balance(banks_client: &mut BanksClient, pubkey: Pubkey) -> u64 {
    get_token(banks_client, pubkey).await.amount
}

pub async fn get_escrow(banks_client: &mut BanksClient, pubkey: Pubkey) -> Escrow {
    let account = get_account(banks_client, pubkey).await;
    Escrow::unpack(&account.data).unwrap()
}

pub async fn get_job(banks_client: &mut BanksClient, pubkey: Pubkey) -> Job {
    let account = get_account(banks_client, pubkey).await;
    Job::unpack(&account.data).unwrap()
}

pub struct TestMint {
    pub pubkey: Pubkey,
    pub authority: Keypair,
    pub decimals: u8,
}

impl TestMint {
    pub fn add(test: &mut ProgramTest, decimals: u8, supply: u64) -> Self {
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

        Self {
            pubkey,
            authority,
            decimals,
        }
    }

    pub async fn get(&self, banks_client: &mut BanksClient) -> Mint {
        get_mint(banks_client, self.pubkey).await
    }
}

pub struct TestToken {
    pub pubkey: Pubkey,
    pub mint: Pubkey,
    pub owner: Keypair,
}

impl TestToken {
    pub fn add(test: &mut ProgramTest, mint: Pubkey, amount: u64) -> Self {
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

        Self {
            pubkey,
            mint,
            owner,
        }
    }

    pub async fn get(&self, banks_client: &mut BanksClient) -> Token {
        get_token(banks_client, self.pubkey).await
    }
}

pub struct TestEscrow {
    pub pubkey: Pubkey,
    pub associated_token: Pubkey,
    pub owner: Keypair,
}

impl TestEscrow {
    pub fn add(test: &mut ProgramTest, token_mint: Pubkey, amount: u64) -> Self {
        let owner = Keypair::new();

        let (pubkey, _bump_seed) = Pubkey::find_program_address(
            &[b"escrow", token_mint.as_ref(), spl_token::id().as_ref()],
            &rndr::id(),
        );

        let associated_token = get_associated_token_address(&pubkey, &token_mint);

        test.add_packable_account(
            associated_token,
            u32::MAX as u64,
            &Token {
                mint: token_mint,
                owner: pubkey,
                amount,
                state: AccountState::Initialized,
                is_native: COption::None,
                ..Token::default()
            },
            &spl_token::id(),
        );

        let mut escrow = Escrow::new(InitEscrowParams {
            owner: owner.pubkey(),
        });
        escrow.amount = amount;
        test.add_packable_account(pubkey, u32::MAX as u64, &escrow, &rndr::id());

        Self {
            pubkey,
            associated_token,
            owner,
        }
    }

    pub async fn get(&self, banks_client: &mut BanksClient) -> Escrow {
        get_escrow(banks_client, self.pubkey).await
    }
}

pub struct TestJob {
    pub pubkey: Pubkey,
    pub authority: Pubkey,
}

impl TestJob {
    pub fn add(test: &mut ProgramTest, escrow: Pubkey, authority: Pubkey, amount: u64) -> Self {
        let (pubkey, _bump_seed) = Pubkey::find_program_address(
            &[b"job", &escrow.as_ref(), &authority.as_ref()],
            &rndr::id(),
        );

        let mut job = Job::new(InitJobParams { authority });
        job.amount = amount;
        test.add_packable_account(pubkey, u32::MAX as u64, &job, &rndr::id());

        Self { pubkey, authority }
    }

    pub async fn get(&self, banks_client: &mut BanksClient) -> Job {
        get_job(banks_client, self.pubkey).await
    }
}
