use anchor_lang::prelude::*;
// use anchor_spl::token::Token; // Removed unused import
use anchor_spl::token::{Mint, TokenAccount}; 
use crate::state::*;
use crate::errors::*;

pub const PERENA_USD_STAR_MINT_PUBKEY_STR: &str = "BenJy1n3WTx9mTjEvy63e8Q1j4RqUc6E4VBMz3ir4Wo6";

#[derive(Accounts)]
pub struct InitializeConfigAccounts<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + 32 + 32 + 32 + 8 + 2 + 1 + 1, 
        seeds = [b"marketplace_config".as_ref()],
        bump
    )]
    pub marketplace_config: Account<'info, MarketplaceConfig>,
    
    #[account(
        constraint = pern_usd_star_mint.key() == PERENA_USD_STAR_MINT_PUBKEY_STR.parse::<Pubkey>().unwrap() @ DubuuMarketplaceError::InvalidPerenaMint
    )]
    pub pern_usd_star_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateConfigAccounts<'info> {
    #[account(
        mut,
        seeds = [b"marketplace_config".as_ref()],
        bump = marketplace_config.bump,
        has_one = admin @ DubuuMarketplaceError::Unauthorized
    )]
    pub marketplace_config: Account<'info, MarketplaceConfig>,
    
    pub admin: Signer<'info>,
}

pub fn handle_initialize_config(
    ctx: Context<InitializeConfigAccounts>,
    admin: Pubkey,
    treasury_pern_account: Pubkey,
    listing_fee_usd_star: u64,
    sale_commission_bps: u16,
) -> Result<()> {
    let marketplace_config = &mut ctx.accounts.marketplace_config;
    marketplace_config.admin = admin;
    marketplace_config.pern_usd_star_mint = ctx.accounts.pern_usd_star_mint.key();
    marketplace_config.treasury_pern_account = treasury_pern_account;
    marketplace_config.listing_fee_usd_star = listing_fee_usd_star;
    marketplace_config.sale_commission_bps = sale_commission_bps;
    marketplace_config.is_paused = false;
    marketplace_config.bump = ctx.bumps.marketplace_config;
    
    Ok(())
}

pub fn handle_update_config(
    ctx: Context<UpdateConfigAccounts>,
    new_listing_fee: Option<u64>,
    new_sale_commission: Option<u16>,
    new_treasury: Option<Pubkey>,
    new_paused_status: Option<bool>,
    new_admin: Option<Pubkey>,
) -> Result<()> {
    let marketplace_config = &mut ctx.accounts.marketplace_config;
    
    if let Some(fee) = new_listing_fee {
        marketplace_config.listing_fee_usd_star = fee;
    }
    
    if let Some(commission) = new_sale_commission {
        marketplace_config.sale_commission_bps = commission;
    }
    
    if let Some(treasury) = new_treasury {
        marketplace_config.treasury_pern_account = treasury;
    }
    
    if let Some(paused) = new_paused_status {
        marketplace_config.is_paused = paused;
    }
    
    if let Some(admin) = new_admin {
        marketplace_config.admin = admin;
    }
    
    Ok(())
}
