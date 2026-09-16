pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("HMqTrRBxW996utybyhDGYcxRvhNtGYqiYzHx56k2zoFA");

#[program]
pub mod amm_q3_2026 {
    use super::*;

    pub fn initalize(ctx : Context<Initialize>,seed : u64,fee : u16,authority : Option<Pubkey>) -> Result<()>{
        ctx.accounts.init(seed, fee, authority, ctx.bumps)
    }
}
