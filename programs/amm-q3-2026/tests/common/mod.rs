#![allow(dead_code)]

use amm_q3_2026::state::Config;
use anchor_lang::{prelude::AccountDeserialize, InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use litesvm_token::{
    get_spl_account, spl_token::state::Account as SplTokenAccount, CreateAssociatedTokenAccount,
    CreateMint, MintTo,
};
use solana_address::Address;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use std::path::PathBuf;

pub const SEED: u64 = 1;
pub const FEE: u16 = 30;
pub const INITIAL_USER_BALANCE: u64 = 10_000_000_000;
pub const INITIAL_LP: u64 = 1_000_000_000;
pub const INITIAL_X: u64 = 1_000_000_000;
pub const INITIAL_Y: u64 = 1_000_000_000;

pub struct TestEnv {
    pub svm: LiteSVM,
    pub payer: Keypair,
    pub mint_x: Address,
    pub mint_y: Address,
    pub config: Pubkey,
    pub mint_lp: Pubkey,
    pub vault_x: Pubkey,
    pub vault_y: Pubkey,
    pub treasury_x: Pubkey,
    pub treasury_y: Pubkey,
    pub user_x: Address,
    pub user_y: Address,
}

pub fn to_address(pk: Pubkey) -> Address {
    Address::new_from_array(pk.to_bytes())
}

pub fn to_pubkey(addr: Address) -> Pubkey {
    Pubkey::new_from_array(addr.to_bytes())
}

pub fn token_amount(svm: &LiteSVM, account: &Address) -> u64 {
    let acc: SplTokenAccount = get_spl_account(svm, account).unwrap();
    acc.amount
}

fn program_so() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/amm_q3_2026.so")
}

impl TestEnv {
    pub fn setup() -> Self {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

        let so = program_so();
        assert!(
            so.exists(),
            "program .so missing at {} — run `anchor build` first",
            so.display()
        );
        svm.add_program_from_file(amm_q3_2026::ID, so).unwrap();

        let mint_x = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .send()
            .unwrap();
        let mint_y = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .send()
            .unwrap();

        let user_x = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_x)
            .send()
            .unwrap();
        let user_y = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_y)
            .send()
            .unwrap();

        MintTo::new(&mut svm, &payer, &mint_x, &user_x, INITIAL_USER_BALANCE)
            .send()
            .unwrap();
        MintTo::new(&mut svm, &payer, &mint_y, &user_y, INITIAL_USER_BALANCE)
            .send()
            .unwrap();

        let program_id = amm_q3_2026::ID;
        let (config, _) =
            Pubkey::find_program_address(&[b"config", &SEED.to_le_bytes()], &program_id);
        let (mint_lp, _) = Pubkey::find_program_address(&[b"lp", config.as_ref()], &program_id);
        let (treasury_x, _) =
            Pubkey::find_program_address(&[b"treasury_x", config.as_ref()], &program_id);
        let (treasury_y, _) =
            Pubkey::find_program_address(&[b"treasury_y", config.as_ref()], &program_id);

        let vault_x = Pubkey::find_program_address(
            &[
                config.as_ref(),
                anchor_spl::token::ID.as_ref(),
                to_pubkey(mint_x).as_ref(),
            ],
            &anchor_spl::associated_token::ID,
        )
        .0;
        let vault_y = Pubkey::find_program_address(
            &[
                config.as_ref(),
                anchor_spl::token::ID.as_ref(),
                to_pubkey(mint_y).as_ref(),
            ],
            &anchor_spl::associated_token::ID,
        )
        .0;

        Self {
            svm,
            payer,
            mint_x,
            mint_y,
            config,
            mint_lp,
            vault_x,
            vault_y,
            treasury_x,
            treasury_y,
            user_x,
            user_y,
        }
    }

    pub fn initialized() -> Self {
        let mut env = Self::setup();
        env.initialize_pool();
        env
    }

    pub fn try_send(&mut self, ix: Instruction) -> Result<(), litesvm::types::FailedTransactionMetadata> {
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            self.svm.latest_blockhash(),
        );
        self.svm.send_transaction(tx).map(|_| ())
    }

    pub fn send(&mut self, ix: Instruction) {
        self.try_send(ix).unwrap_or_else(|err| {
            panic!("transaction failed: {err:?}");
        });
    }

    pub fn initialize_pool(&mut self) {
        let ix = Instruction {
            program_id: amm_q3_2026::ID,
            accounts: amm_q3_2026::accounts::Initialize {
                initializer: to_pubkey(self.payer.pubkey()),
                mint_x: to_pubkey(self.mint_x),
                mint_y: to_pubkey(self.mint_y),
                mint_lp: self.mint_lp,
                vault_x: self.vault_x,
                vault_y: self.vault_y,
                treasury_x: self.treasury_x,
                treasury_y: self.treasury_y,
                config: self.config,
                token_program: anchor_spl::token::ID,
                associated_token_program: anchor_spl::associated_token::ID,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
            data: amm_q3_2026::instruction::Initalize {
                seed: SEED,
                fee: FEE,
                authority: Some(to_pubkey(self.payer.pubkey())),
            }
            .data(),
        };
        self.send(ix);
    }

    pub fn deposit_ix(&self, amount: u64, max_x: u64, max_y: u64) -> Instruction {
        let user = to_pubkey(self.payer.pubkey());
        let mint_lp = self.mint_lp;
        let user_lp = Pubkey::find_program_address(
            &[
                user.as_ref(),
                anchor_spl::token::ID.as_ref(),
                mint_lp.as_ref(),
            ],
            &anchor_spl::associated_token::ID,
        )
        .0;
        Instruction {
            program_id: amm_q3_2026::ID,
            accounts: amm_q3_2026::accounts::Deposit {
                user,
                mint_x: to_pubkey(self.mint_x),
                mint_y: to_pubkey(self.mint_y),
                config: self.config,
                mint_lp,
                vault_x: self.vault_x,
                vault_y: self.vault_y,
                user_x: to_pubkey(self.user_x),
                user_y: to_pubkey(self.user_y),
                user_lp,
                token_program: anchor_spl::token::ID,
                system_program: anchor_lang::system_program::ID,
                associated_token_program: anchor_spl::associated_token::ID,
            }
            .to_account_metas(None),
            data: amm_q3_2026::instruction::Deposit {
                amount,
                max_x,
                max_y,
            }
            .data(),
        }
    }

    pub fn deposit(&mut self, amount: u64, max_x: u64, max_y: u64) {
        let ix = self.deposit_ix(amount, max_x, max_y);
        self.send(ix);
    }

    pub fn swap(&mut self, is_x: bool, amount_in: u64, min_amount_out: u64) {
        let ix = Instruction {
            program_id: amm_q3_2026::ID,
            accounts: amm_q3_2026::accounts::Swap {
                user: to_pubkey(self.payer.pubkey()),
                mint_x: to_pubkey(self.mint_x),
                mint_y: to_pubkey(self.mint_y),
                config: self.config,
                mint_lp: self.mint_lp,
                vault_x: self.vault_x,
                vault_y: self.vault_y,
                user_x: to_pubkey(self.user_x),
                user_y: to_pubkey(self.user_y),
                treasury_x: self.treasury_x,
                treasury_y: self.treasury_y,
                token_program: anchor_spl::token::ID,
                system_program: anchor_lang::system_program::ID,
                associated_token_program: anchor_spl::associated_token::ID,
            }
            .to_account_metas(None),
            data: amm_q3_2026::instruction::Swap {
                is_x,
                amount_in,
                min_amount_out,
            }
            .data(),
        };
        self.send(ix);
    }

    pub fn withdraw(&mut self, amount: u64, min_x: u64, min_y: u64) {
        let user = to_pubkey(self.payer.pubkey());
        let mint_lp = self.mint_lp;
        let user_lp = Pubkey::find_program_address(
            &[
                user.as_ref(),
                anchor_spl::token::ID.as_ref(),
                mint_lp.as_ref(),
            ],
            &anchor_spl::associated_token::ID,
        )
        .0;
        let ix = Instruction {
            program_id: amm_q3_2026::ID,
            accounts: amm_q3_2026::accounts::Withdraw {
                user,
                mint_x: to_pubkey(self.mint_x),
                mint_y: to_pubkey(self.mint_y),
                config: self.config,
                mint_lp,
                vault_x: self.vault_x,
                vault_y: self.vault_y,
                user_x: to_pubkey(self.user_x),
                user_y: to_pubkey(self.user_y),
                user_lp,
                token_program: anchor_spl::token::ID,
                system_program: anchor_lang::system_program::ID,
                associated_token_program: anchor_spl::associated_token::ID,
            }
            .to_account_metas(None),
            data: amm_q3_2026::instruction::Withdraw {
                amount,
                min_x,
                min_y,
            }
            .data(),
        };
        self.send(ix);
    }

    pub fn config_account(&self) -> Config {
        let data = self.svm.get_account(&to_address(self.config)).unwrap().data;
        Config::try_deserialize(&mut data.as_slice()).unwrap()
    }

    pub fn payer_pk(&self) -> Pubkey {
        to_pubkey(self.payer.pubkey())
    }

    pub fn user_lp(&self) -> Pubkey {
        let user = self.payer_pk();
        Pubkey::find_program_address(
            &[
                user.as_ref(),
                anchor_spl::token::ID.as_ref(),
                self.mint_lp.as_ref(),
            ],
            &anchor_spl::associated_token::ID,
        )
        .0
    }
}
