use anchor_lang::prelude::*;

pub mod config_module;
pub mod asset_module;
pub mod auction_module;
pub mod state;
pub mod errors;

use config_module::*;
use asset_module::*;
use auction_module::*;
use state::*;

declare_id!("3sfrzZ8nqttybB1egP7Sep1LxhHuPq7bmKg7v9TrYd9W");

#[programs]
pub mod dubuu_marketplace_mvp {
    use super::*;
    // Config Module Instructions
    pub fn initialize_config(
        ctx: Context<InitializeConfigAccounts>,
        admin: Pubkey,
        treasury_pern_account: Pubkey,
        listing_fee_usd_star: u64,
        sale_commission_bps: u16,
    ) -> Result<()> {
        config_module::handle_initialize_config(ctx, admin, treasury_pern_account, listing_fee_usd_star, sale_commission_bps)
    }

    pub fn update_config(
        ctx: Context<UpdateConfigAccounts>,
        new_listing_fee: Option<u64>,
        new_sale_commission: Option<u16>,
        new_treasury: Option<Pubkey>,
        new_paused_status: Option<bool>,
        new_admin: Option<Pubkey>,
    ) -> Result<()> {
        config_module::handle_update_config(ctx, new_listing_fee, new_sale_commission, new_treasury, new_paused_status, new_admin)
    }

    // Asset Module Instructions
    pub fn register_asset_and_submit_docs_ref(
        ctx: Context<RegisterAssetAccounts>,
        asset_id_seed_str: String,
        walrus_main_metadata_cid: String,
    ) -> Result<()> {
        asset_module::handle_register_asset_and_submit_docs_ref(ctx, asset_id_seed_str, walrus_main_metadata_cid)
    }

    pub fn admin_update_ownership_verification(
        ctx: Context<AdminVerifyOwnershipAccounts>,
        new_verification_status: OwnershipStatus,
        verification_notes_hash: Option<[u8; 32]>,
    ) -> Result<()> {
        asset_module::handle_admin_update_ownership_verification(ctx, new_verification_status, verification_notes_hash)
    }

    pub fn update_asset_walrus_cid(
        ctx: Context<UpdateAssetMetadataAccounts>,
        new_walrus_main_metadata_cid: String,
    ) -> Result<()> {
        asset_module::handle_update_asset_walrus_cid(ctx, new_walrus_main_metadata_cid)
    }

    // Auction Module Instructions
    pub fn initialize_wormhole_listener(
        ctx: Context<InitializeWormholeListenerAccounts>,
        authorized_relayer: Pubkey,
    ) -> Result<()> {
        auction_module::handle_initialize_wormhole_listener(ctx, authorized_relayer)
    }

    pub fn list_asset_for_auction(
        ctx: Context<ListAssetForAuctionAccounts>,
        start_price_usd_star: u64,
        duration_seconds: i64,
    ) -> Result<()> {
        auction_module::handle_list_asset_for_auction(ctx, start_price_usd_star, duration_seconds)
    }

    pub fn place_bid(
        ctx: Context<PlaceBidAccounts>,
        bid_amount_usd_star: u64,
    ) -> Result<()> {
        auction_module::handle_place_bid(ctx, bid_amount_usd_star)
    }

    pub fn finalize_auction(
        ctx: Context<FinalizeAuctionAccounts>,
    ) -> Result<()> {
        auction_module::handle_finalize_auction(ctx)
    }

    pub fn settle_auction_and_transfer(
        ctx: Context<SettleAuctionAccounts>,
    ) -> Result<()> {
        auction_module::handle_settle_auction_and_transfer(ctx)
    }

    pub fn process_wormhole_balance_attestation(
        ctx: Context<ProcessWormholeAttestationAccounts>,
        payload: BalanceAttestationPayload,
    ) -> Result<()> {
        auction_module::handle_process_wormhole_balance_attestation(ctx, payload)
    }
}
