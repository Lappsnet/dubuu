use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash; // For hashing asset_id_seed_str
use crate::state::*; // Assumes AssetAccount, OwnershipStatus, AssetListedStatus, MarketplaceConfig are here
use crate::errors::*; // Assumes DubuuMarketplaceError is here

// Define the maximum length for the CID string
// Adjust this value based on the typical length of your Walrus CIDs (e.g., IPFS v0 or v1)
const MAX_METADATA_CID_LENGTH: usize = 100; // Example: IPFS CIDs are typically around 46-59 chars for v0/v1

// --- Account Context Structs for Instructions ---

#[derive(Accounts)]
#[instruction(asset_id_seed_str: String, walrus_main_metadata_cid: String)]
pub struct RegisterAssetAccounts<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + 32 + 32 + 32 + (4 + MAX_METADATA_CID_LENGTH) + 1 + 1 + (1 + 32) + 1,
        seeds = [
            b"asset",
            &hash::hash(asset_id_seed_str.as_bytes()).to_bytes()[..5] // Take first 5 bytes
        ],
        bump
    )]
    pub asset_account: Account<'info, AssetAccount>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminVerifyOwnershipAccounts<'info> {
    #[account(mut)]
    pub asset_account: Account<'info, AssetAccount>,

    #[account(
        seeds = [b"marketplace_config"], // Assuming MarketplaceConfig is a singleton PDA
        bump = marketplace_config.bump
    )]
    pub marketplace_config: Account<'info, MarketplaceConfig>,

    #[account(
        constraint = admin.key() == marketplace_config.admin @ DubuuMarketplaceError::Unauthorized
    )]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateAssetMetadataAccounts<'info> {
    #[account(
        mut,
        has_one = current_owner @ DubuuMarketplaceError::Unauthorized // Ensures signer is the current owner
    )]
    pub asset_account: Account<'info, AssetAccount>,

    pub current_owner: Signer<'info>, // The signer must be the current_owner
}

// REMOVED or REPURPOSED: This context might not be directly used by the refactored internal functions.
// If other true "internal instructions" within this module need a similar restricted context,
// you could define a new one. For now, the refactored functions below take direct account refs.
//
// #[derive(Accounts)]
// pub struct InternalAssetMutationAccounts<'info> { // Example of a more generic internal context
//     #[account(mut)]
//     pub asset_account: Account<'info, AssetAccount>,
//     // Potentially add a program_signer or other constraints if these were actual CPIs
// }


// --- Instruction Handlers (exposed in lib.rs #[program] block) ---

pub fn handle_register_asset_and_submit_docs_ref(
    ctx: Context<RegisterAssetAccounts>,
    asset_id_seed_str: String, // This is used by Anchor for PDA derivation if specified in #[instruction]
    walrus_main_metadata_cid: String,
) -> Result<()> {
    require!(
        walrus_main_metadata_cid.len() <= MAX_METADATA_CID_LENGTH,
        DubuuMarketplaceError::StringTooLong
    );
    // It's good practice to also validate asset_id_seed_str length if it has constraints

    let asset_account = &mut ctx.accounts.asset_account;
    let bump = ctx.bumps.asset_account; // Get the bump for the asset_account PDA

    let asset_id_hash = hash::hash(asset_id_seed_str.as_bytes()).to_bytes();

    asset_account.creator = ctx.accounts.signer.key();
    asset_account.current_owner = ctx.accounts.signer.key();
    asset_account.asset_id_hash = asset_id_hash;
    asset_account.walrus_main_metadata_cid = walrus_main_metadata_cid;
    asset_account.ownership_verification_status = OwnershipStatus::PendingReview;
    asset_account.asset_listed_status = AssetListedStatus::AwaitingOwnershipVerification;
    asset_account.active_auction_key = None;
    asset_account.bump = bump;

    // Optionally emit an event for asset registration
    // emit!(AssetRegistered { asset_key: asset_account.key(), creator: asset_account.creator });

    Ok(())
}

pub fn handle_admin_update_ownership_verification(
    ctx: Context<AdminVerifyOwnershipAccounts>,
    new_verification_status: OwnershipStatus,
    verification_notes_hash: Option<[u8; 32]>, // Optional hash of off-chain verification notes
) -> Result<()> {
    let asset_account = &mut ctx.accounts.asset_account;

    asset_account.ownership_verification_status = new_verification_status.clone();

    if new_verification_status == OwnershipStatus::Verified {
        asset_account.asset_listed_status = AssetListedStatus::ReadyForAuction;
    } else if new_verification_status == OwnershipStatus::Rejected {
        // Optional: Reset asset_listed_status or set to a specific "rejected" state if needed
        // For now, if rejected, it remains in AwaitingOwnershipVerification or similar until resubmitted/updated.
        // Or, you might want to set it back to Unlisted if appropriate.
        // asset_account.asset_listed_status = AssetListedStatus::Unlisted;
    }

    emit!(OwnershipVerificationUpdatedEvent {
        asset_key: asset_account.key(),
        status: new_verification_status, // The new_verification_status.clone() is not needed for the event if the original is consumed or not needed after
        notes_hash: verification_notes_hash,
    });

    Ok(())
}

pub fn handle_update_asset_walrus_cid(
    ctx: Context<UpdateAssetMetadataAccounts>,
    new_walrus_main_metadata_cid: String,
) -> Result<()> {
    require!(
        new_walrus_main_metadata_cid.len() <= MAX_METADATA_CID_LENGTH,
        DubuuMarketplaceError::StringTooLong
    );

    let asset_account = &mut ctx.accounts.asset_account;

    // Ensure asset is not in an active auction or already sold
    require!(
        asset_account.asset_listed_status != AssetListedStatus::InAuction &&
        asset_account.asset_listed_status != AssetListedStatus::Sold,
        DubuuMarketplaceError::AssetStatusPreventsUpdate
    );
    // Add more checks if needed, e.g., based on ownership_verification_status

    asset_account.walrus_main_metadata_cid = new_walrus_main_metadata_cid;

    // Optionally, if metadata update requires re-verification:
    // asset_account.ownership_verification_status = OwnershipStatus::PendingReview;
    // asset_account.asset_listed_status = AssetListedStatus::AwaitingOwnershipVerification;
    // emit!(AssetMetadataUpdatedForReverification { asset_key: asset_account.key() });

    Ok(())
}


// --- Public Helper Functions (called by other modules within the same program, NOT direct instructions) ---

// UPDATED: Changed signature - no longer takes Context, takes mutable AssetAccount directly.
// Renamed from handle_internal_transfer_ownership.
pub fn internal_transfer_ownership<'info>(
    asset_account: &mut Account<'info, AssetAccount>, // Passed directly from calling module
    new_owner: Pubkey,
) -> Result<()> {
    // Internal business logic checks can be added here if necessary,
    // though primary authorization should happen in the calling instruction (e.g., settle_auction).

    asset_account.current_owner = new_owner;
    asset_account.asset_listed_status = AssetListedStatus::Sold;
    asset_account.active_auction_key = None; // Clear any active auction link

    emit!(AssetSold {
        asset_key: asset_account.key(),
        new_owner, // new_owner is already a Pubkey
        walrus_main_metadata_cid: asset_account.walrus_main_metadata_cid.clone(),
    });

    Ok(())
}

// UPDATED: Changed signature - no longer takes Context, takes mutable AssetAccount directly.
// Renamed from handle_internal_update_asset_status_to_in_auction.
pub fn internal_update_asset_status_to_in_auction<'info>(
    asset_account: &mut Account<'info, AssetAccount>, // Passed directly from calling module
    auction_key: Pubkey,
) -> Result<()> {
    // This function assumes that the calling function (e.g., list_asset_for_auction)
    // has already verified that the asset *can* be moved to 'InAuction' state.

    require!(
        asset_account.asset_listed_status == AssetListedStatus::ReadyForAuction,
        DubuuMarketplaceError::AssetNotReadyForAuction
    );
    require!(
        asset_account.ownership_verification_status == OwnershipStatus::Verified,
        DubuuMarketplaceError::OwnershipVerificationRequired
    );

    asset_account.asset_listed_status = AssetListedStatus::InAuction;
    asset_account.active_auction_key = Some(auction_key);

    // Optionally emit an event
    // emit!(AssetListedForAuction { asset_key: asset_account.key(), auction_key: auction_key });

    Ok(())
}

// Ensure your state.rs or this file defines these:
// #[account] pub struct AssetAccount { ... }
// pub enum OwnershipStatus { ... }
// pub enum AssetListedStatus { ... }
// #[event] pub struct OwnershipVerificationUpdatedEvent { ... }
// #[event] pub struct AssetSold { ... }

// Ensure your errors.rs defines these:
// enum DubuuMarketplaceError { Unauthorized, StringTooLong, AssetStatusPreventsUpdate, ... }