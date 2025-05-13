use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Mint, Transfer, CloseAccount};
// Removed: use anchor_spl::associated_token::AssociatedToken;
use crate::state::*; // This will bring in BalanceAttestationPayload with the correct field name
use crate::errors::*;
use crate::asset_module;

// ASSUMED UPDATED SIGNATURES in asset_module.rs for direct calls:
// pub fn internal_update_asset_status_to_in_auction<'info>(
//     asset_account: &mut Account<'info, AssetAccount>,
//     auction_key: Pubkey,
// ) -> Result<()>
//
// pub fn internal_transfer_ownership<'info>(
//     asset_account: &mut Account<'info, AssetAccount>,
//     new_owner: Pubkey,
// ) -> Result<()>


#[derive(Accounts)]
pub struct InitializeWormholeListenerAccounts<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + 32 + 1, 
        seeds = [b"wormhole_listener".as_ref()],
        bump
    )]
    pub wormhole_listener_config: Account<'info, WormholeListenerConfig>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ListAssetForAuctionAccounts<'info> {
    #[account(
        init,
        payer = seller,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 33 + 1 + 1 + 1,
        seeds = [b"auction".as_ref(), asset_account.key().as_ref()],
        bump
    )]
    pub auction_account: Account<'info, AuctionAccount>,
    
    #[account(
        mut,
        constraint = asset_account.current_owner == seller.key() @ DubuuMarketplaceError::Unauthorized,
        constraint = asset_account.ownership_verification_status == OwnershipStatus::Verified @ DubuuMarketplaceError::OwnershipVerificationRequired,
        constraint = asset_account.asset_listed_status == AssetListedStatus::ReadyForAuction @ DubuuMarketplaceError::AssetNotReadyForAuction
    )]
    pub asset_account: Account<'info, AssetAccount>,
    
    #[account(mut)]
    pub seller: Signer<'info>,
    
    #[account(
        seeds = [b"marketplace_config".as_ref()],
        bump = marketplace_config.bump,
        constraint = !marketplace_config.is_paused @ DubuuMarketplaceError::MarketplacePaused
    )]
    pub marketplace_config: Account<'info, MarketplaceConfig>,
    
    #[account(
        mut,
        constraint = seller_pern_token_account.mint == marketplace_config.pern_usd_star_mint @ DubuuMarketplaceError::InvalidPerenaMint,
        constraint = seller_pern_token_account.owner == seller.key() @ DubuuMarketplaceError::InvalidTokenAccountOwner
    )]
    pub seller_pern_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        address = marketplace_config.treasury_pern_account @ DubuuMarketplaceError::InvalidTreasuryAccount,
        constraint = treasury_pern_token_account.mint == marketplace_config.pern_usd_star_mint @ DubuuMarketplaceError::InvalidPerenaMint
    )]
    pub treasury_pern_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = seller,
        token::mint = pern_usd_star_mint_account,
        token::authority = auction_escrow_authority,
        seeds = [b"escrow".as_ref(), auction_account.key().as_ref()],
        bump
    )]
    pub auction_escrow_token_account: Account<'info, TokenAccount>,
    
    #[account(
        address = marketplace_config.pern_usd_star_mint @ DubuuMarketplaceError::InvalidPerenaMint
    )]
    pub pern_usd_star_mint_account: Account<'info, Mint>,
    
    /// CHECK: This is a PDA that will be the authority for the escrow account's tokens.
    #[account(
        seeds = [b"escrow_authority".as_ref(), auction_account.key().as_ref()],
        bump
    )]
    pub auction_escrow_authority: AccountInfo<'info>,
    
    pub token_program: Program<'info, token::Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBidAccounts<'info> {
    #[account(
        mut,
        seeds = [b"auction".as_ref(), auction_account.asset_key.as_ref()],
        bump = auction_account.bump,
        constraint = auction_account.auction_status == AuctionProcessStatus::Active @ DubuuMarketplaceError::AuctionNotInActiveState
    )]
    pub auction_account: Account<'info, AuctionAccount>,
    
    #[account(mut)]
    pub bidder: Signer<'info>,
    
    #[account(
        mut,
        constraint = bidder_pern_token_account.mint == auction_account.pern_usd_star_mint @ DubuuMarketplaceError::InvalidPerenaMint,
        constraint = bidder_pern_token_account.owner == bidder.key() @ DubuuMarketplaceError::InvalidTokenAccountOwner
    )]
    pub bidder_pern_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"escrow".as_ref(), auction_account.key().as_ref()],
        bump
    )]
    pub auction_escrow_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is the PDA authority for the escrow account's tokens.
    #[account(
        seeds = [b"escrow_authority".as_ref(), auction_account.key().as_ref()],
        bump = auction_account.escrow_authority_bump
    )]
    pub auction_escrow_authority: AccountInfo<'info>,
    
    #[account(mut)]
    pub previous_highest_bidder_token_account: Option<Account<'info, TokenAccount>>,
    
    pub token_program: Program<'info, token::Token>,
}

#[derive(Accounts)]
pub struct FinalizeAuctionAccounts<'info> {
    #[account(
        mut,
        seeds = [b"auction".as_ref(), auction_account.asset_key.as_ref()],
        bump = auction_account.bump
    )]
    pub auction_account: Account<'info, AuctionAccount>,

    pub signer: Signer<'info>, 

    #[account(
        mut,
        seeds = [b"escrow".as_ref(), auction_account.key().as_ref()],
        bump
    )]
    pub auction_escrow_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA authority for the escrow account's tokens.
    #[account(
        seeds = [b"escrow_authority".as_ref(), auction_account.key().as_ref()],
        bump = auction_account.escrow_authority_bump
    )]
    pub auction_escrow_authority: AccountInfo<'info>,
    
    #[account(mut)]
    pub highest_bidder_token_account_for_refund: Option<Account<'info, TokenAccount>>, 
    
    /// CHECK: Seller's account to return rent to when closing escrow if unsold with no bids.
    #[account(mut, address = auction_account.seller @ DubuuMarketplaceError::InvalidSellerAccountForRent)]
    pub seller_rent_recipient: AccountInfo<'info>,

    pub token_program: Program<'info, token::Token>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct SettleAuctionAccounts<'info> {
    #[account(
        mut,
        seeds = [b"auction".as_ref(), auction_account.asset_key.as_ref()],
        bump = auction_account.bump,
        constraint = auction_account.auction_status == AuctionProcessStatus::EndedSoldPayPending @ DubuuMarketplaceError::AuctionNotInSettlementState,
        constraint = auction_account.highest_bidder.is_some() && auction_account.highest_bidder.unwrap() == highest_bidder.key() @ DubuuMarketplaceError::NotAuctionWinner
    )]
    pub auction_account: Account<'info, AuctionAccount>,
    
    #[account(mut)]
    pub highest_bidder: Signer<'info>,
    
    #[account(
        mut,
        constraint = asset_account.key() == auction_account.asset_key @ DubuuMarketplaceError::InvalidAssetAccount
    )]
    pub asset_account: Account<'info, AssetAccount>,
    
    #[account(
        seeds = [b"marketplace_config".as_ref()],
        bump = marketplace_config.bump
    )]
    pub marketplace_config: Account<'info, MarketplaceConfig>,
    
    #[account(
        mut,
        seeds = [b"escrow".as_ref(), auction_account.key().as_ref()],
        bump
    )]
    pub auction_escrow_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: PDA authority for the escrow account's tokens.
    #[account(
        seeds = [b"escrow_authority".as_ref(), auction_account.key().as_ref()],
        bump = auction_account.escrow_authority_bump
    )]
    pub auction_escrow_authority: AccountInfo<'info>,
    
    #[account(
        mut,
        constraint = seller_token_account.owner == auction_account.seller @ DubuuMarketplaceError::InvalidTokenAccountOwner,
        constraint = seller_token_account.mint == auction_account.pern_usd_star_mint @ DubuuMarketplaceError::InvalidPerenaMint
    )]
    pub seller_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        address = marketplace_config.treasury_pern_account @ DubuuMarketplaceError::InvalidTreasuryAccount,
        constraint = treasury_pern_token_account.mint == auction_account.pern_usd_star_mint @ DubuuMarketplaceError::InvalidPerenaMint
    )]
    pub treasury_pern_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, token::Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(payload_argument: BalanceAttestationPayload)] // This makes payload_argument available to seeds/constraints
pub struct ProcessWormholeAttestationAccounts<'info> {
    #[account(
        init_if_needed,
        payer = relayer,
        space = 8 + 32 + 2 + 32 + 8 + 8 + 1, 
        seeds = [
            b"attestation".as_ref(), 
            payload_argument.solana_target_address.as_ref(), 
            &payload_argument.evm_chain_id.to_le_bytes(),
            // This uses the field from the #[instruction] argument
            &anchor_lang::solana_program::hash::hash(&payload_argument.asset_address_on_evm_as_bytes).to_bytes()
        ],
        bump
    )]
    pub cross_chain_attestation: Account<'info, CrossChainAttestation>,
    
    #[account(
        seeds = [b"wormhole_listener".as_ref()],
        bump = wormhole_listener_config.bump,
        constraint = relayer.key() == wormhole_listener_config.wormhole_authorized_relayer @ DubuuMarketplaceError::Unauthorized
    )]
    pub wormhole_listener_config: Account<'info, WormholeListenerConfig>,
    
    #[account(mut)]
    pub relayer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}


// --- Instruction Handlers ---

pub fn handle_initialize_wormhole_listener(
    ctx: Context<InitializeWormholeListenerAccounts>,
    authorized_relayer: Pubkey,
) -> Result<()> {
    let wormhole_listener_config = &mut ctx.accounts.wormhole_listener_config;
    wormhole_listener_config.wormhole_authorized_relayer = authorized_relayer;
    wormhole_listener_config.bump = ctx.bumps.wormhole_listener_config;
    
    Ok(())
}

pub fn handle_list_asset_for_auction(
    ctx: Context<ListAssetForAuctionAccounts>,
    start_price_usd_star: u64,
    duration_seconds: i64,
) -> Result<()> {
    let auction_account = &mut ctx.accounts.auction_account;
    let marketplace_config = &ctx.accounts.marketplace_config;
    let asset_account = &mut ctx.accounts.asset_account;

    let auction_account_bump = ctx.bumps.auction_account;
    let auction_escrow_authority_bump = ctx.bumps.auction_escrow_authority;
    
    let cpi_accounts_fee = Transfer {
        from: ctx.accounts.seller_pern_token_account.to_account_info(),
        to: ctx.accounts.treasury_pern_token_account.to_account_info(),
        authority: ctx.accounts.seller.to_account_info(),
    };
    let cpi_ctx_fee = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts_fee,
    );
    token::transfer(cpi_ctx_fee, marketplace_config.listing_fee_usd_star)?;
    
    let clock = Clock::get()?;
    let auction_end_timestamp = clock.unix_timestamp.checked_add(duration_seconds)
        .ok_or_else(|| DubuuMarketplaceError::TimestampOverflow)?;
    
    auction_account.asset_key = asset_account.key();
    auction_account.seller = ctx.accounts.seller.key();
    auction_account.pern_usd_star_mint = marketplace_config.pern_usd_star_mint;
    auction_account.start_price_usd_star = start_price_usd_star;
    auction_account.auction_end_timestamp = auction_end_timestamp;
    auction_account.highest_bid_usd_star = start_price_usd_star; 
    auction_account.highest_bidder = None;
    auction_account.auction_status = AuctionProcessStatus::Active;
    auction_account.escrow_authority_bump = auction_escrow_authority_bump;
    auction_account.bump = auction_account_bump;
    
    asset_module::internal_update_asset_status_to_in_auction(
        asset_account,
        auction_account.key()
    )?;
    
    Ok(())
}

pub fn handle_place_bid(
    ctx: Context<PlaceBidAccounts>,
    bid_amount_usd_star: u64,
) -> Result<()> {
    let auction_account = &mut ctx.accounts.auction_account;
    
    require!(
        bid_amount_usd_star > auction_account.highest_bid_usd_star,
        DubuuMarketplaceError::BidTooLow
    );
    
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp < auction_account.auction_end_timestamp,
        DubuuMarketplaceError::AuctionEnded
    );
    
    let auction_key_as_bytes = auction_account.key().to_bytes();
    let escrow_authority_seeds_slices: &[&[u8]] = &[
        b"escrow_authority".as_ref(),
        auction_key_as_bytes.as_ref(),
        &[auction_account.escrow_authority_bump],
    ];
    let signer_seeds = &[&escrow_authority_seeds_slices[..]];

    if let Some(previous_highest_bidder_key) = auction_account.highest_bidder {
        let previous_bidder_token_account_opt = ctx.accounts.previous_highest_bidder_token_account.as_ref();
        require!(previous_bidder_token_account_opt.is_some(), DubuuMarketplaceError::MissingPreviousBidderAccount);
        
        let previous_bidder_token_account = previous_bidder_token_account_opt.unwrap();
        require_keys_eq!(previous_bidder_token_account.owner, previous_highest_bidder_key, DubuuMarketplaceError::InvalidTokenAccountOwner);
        require_keys_eq!(previous_bidder_token_account.mint, auction_account.pern_usd_star_mint, DubuuMarketplaceError::InvalidPerenaMint);
        
        let cpi_accounts_return = Transfer {
            from: ctx.accounts.auction_escrow_token_account.to_account_info(),
            to: previous_bidder_token_account.to_account_info(),
            authority: ctx.accounts.auction_escrow_authority.to_account_info(),
        };
        let cpi_ctx_return = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts_return,
            signer_seeds,
        );
        token::transfer(cpi_ctx_return, auction_account.highest_bid_usd_star)?;
    }
    
    let cpi_accounts_new_bid = Transfer {
        from: ctx.accounts.bidder_pern_token_account.to_account_info(),
        to: ctx.accounts.auction_escrow_token_account.to_account_info(),
        authority: ctx.accounts.bidder.to_account_info(),
    };
    let cpi_ctx_new_bid = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts_new_bid,
    );
    token::transfer(cpi_ctx_new_bid, bid_amount_usd_star)?;
    
    auction_account.highest_bidder = Some(ctx.accounts.bidder.key());
    auction_account.highest_bid_usd_star = bid_amount_usd_star;
    
    emit!(BidPlacedEvent {
        auction_key: auction_account.key(),
        bidder: ctx.accounts.bidder.key(),
        amount: bid_amount_usd_star,
    });
    
    Ok(())
}

pub fn handle_finalize_auction( ctx: Context<FinalizeAuctionAccounts>) -> Result<()> {
    let auction_account = &mut ctx.accounts.auction_account;
    
    require!(
        auction_account.auction_status == AuctionProcessStatus::Active,
        DubuuMarketplaceError::AuctionNotInActiveState
    );

    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp >= auction_account.auction_end_timestamp,
        DubuuMarketplaceError::AuctionNotEnded
    );
    
    let auction_key_as_bytes = auction_account.key().to_bytes();
    let escrow_authority_seeds_slices: &[&[u8]] = &[
        b"escrow_authority".as_ref(),
        auction_key_as_bytes.as_ref(),
        &[auction_account.escrow_authority_bump],
    ];
    let signer_seeds = &[&escrow_authority_seeds_slices[..]];

    if auction_account.highest_bidder.is_some() {
        auction_account.auction_status = AuctionProcessStatus::EndedSoldPayPending;
        emit!(AuctionEndedWinner {
            auction_key: auction_account.key(),
            winner: auction_account.highest_bidder.unwrap(),
            winning_bid: auction_account.highest_bid_usd_star,
        });
    } else { 
        auction_account.auction_status = AuctionProcessStatus::EndedUnsold;
        emit!(AuctionEndedNoSale {
            auction_key: auction_account.key(),
        });

        let cpi_accounts_close_escrow = CloseAccount {
            account: ctx.accounts.auction_escrow_token_account.to_account_info(),
            destination: ctx.accounts.seller_rent_recipient.to_account_info(),
            authority: ctx.accounts.auction_escrow_authority.to_account_info(),
        };
        let cpi_ctx_close_escrow = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts_close_escrow,
            signer_seeds,
        );
        token::close_account(cpi_ctx_close_escrow)?;
    }
    
    Ok(())
}

pub fn handle_settle_auction_and_transfer( ctx: Context<SettleAuctionAccounts>) -> Result<()> {
    let auction_account = &mut ctx.accounts.auction_account;
    let marketplace_config = &ctx.accounts.marketplace_config;
    let asset_account = &mut ctx.accounts.asset_account;
    
    let commission_bps = marketplace_config.sale_commission_bps as u64;
    let total_bid_amount = auction_account.highest_bid_usd_star;

    let commission = total_bid_amount
        .checked_mul(commission_bps)
        .ok_or_else(|| DubuuMarketplaceError::CalculationOverflow)?
        .checked_div(10000)
        .ok_or_else(|| DubuuMarketplaceError::CalculationOverflow)?;
        
    let amount_to_seller = total_bid_amount
        .checked_sub(commission)
        .ok_or_else(|| DubuuMarketplaceError::CalculationOverflow)?;
    
    let auction_key_as_bytes = auction_account.key().to_bytes();
    let escrow_authority_seeds_slices: &[&[u8]] = &[
        b"escrow_authority".as_ref(),
        auction_key_as_bytes.as_ref(),
        &[auction_account.escrow_authority_bump],
    ];
    let signer_seeds = &[&escrow_authority_seeds_slices[..]];
    
    if amount_to_seller > 0 {
        let cpi_accounts_to_seller = Transfer {
            from: ctx.accounts.auction_escrow_token_account.to_account_info(),
            to: ctx.accounts.seller_token_account.to_account_info(),
            authority: ctx.accounts.auction_escrow_authority.to_account_info(),
        };
        let cpi_ctx_to_seller = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts_to_seller,
            signer_seeds,
        );
        token::transfer(cpi_ctx_to_seller, amount_to_seller)?;
    }
    
    if commission > 0 {
        let cpi_accounts_to_treasury = Transfer {
            from: ctx.accounts.auction_escrow_token_account.to_account_info(),
            to: ctx.accounts.treasury_pern_token_account.to_account_info(),
            authority: ctx.accounts.auction_escrow_authority.to_account_info(),
        };
        let cpi_ctx_to_treasury = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts_to_treasury,
            signer_seeds,
        );
        token::transfer(cpi_ctx_to_treasury, commission)?;
    }
    
    let cpi_accounts_close_escrow = CloseAccount {
        account: ctx.accounts.auction_escrow_token_account.to_account_info(),
        destination: ctx.accounts.highest_bidder.to_account_info(),
        authority: ctx.accounts.auction_escrow_authority.to_account_info(),
    };
    let cpi_ctx_close_escrow = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts_close_escrow,
        signer_seeds,
    );
    token::close_account(cpi_ctx_close_escrow)?;
    
    asset_module::internal_transfer_ownership(
        asset_account,
        ctx.accounts.highest_bidder.key()
    )?;
    
    auction_account.auction_status = AuctionProcessStatus::Completed;
    
    Ok(())
}

pub fn handle_process_wormhole_balance_attestation(
    ctx: Context<ProcessWormholeAttestationAccounts>,
    payload: BalanceAttestationPayload, // This is the function argument from lib.rs
) -> Result<()> {
    let cross_chain_attestation = &mut ctx.accounts.cross_chain_attestation;
    let bump = ctx.bumps.cross_chain_attestation; 
    
    // The `payload_argument` from #[instruction] is used for seeds.
    // The `payload` function argument (passed from lib.rs) is used here for the logic.
    let hashed_asset_address = anchor_lang::solana_program::hash::hash(&payload.asset_address_on_evm_as_bytes).to_bytes();

    cross_chain_attestation.user_solana_key = payload.solana_target_address;
    cross_chain_attestation.source_chain_id = payload.evm_chain_id;
    cross_chain_attestation.source_asset_hash = hashed_asset_address;
    cross_chain_attestation.attested_balance = payload.balance;
    cross_chain_attestation.attestation_timestamp = payload.timestamp;
    cross_chain_attestation.bump = bump;
    
    emit!(CrossChainBalanceAttestedEvent {
        user_solana_key: payload.solana_target_address,
        source_chain_id: payload.evm_chain_id,
        source_asset_hash: hashed_asset_address,
        attested_balance: payload.balance,
    });
    
    Ok(())
}
