use anchor_lang::prelude::*;

// Config Module State
#[account]
#[derive(Debug)] // Added Debug for easier inspection if needed
pub struct MarketplaceConfig {
    pub admin: Pubkey,
    pub pern_usd_star_mint: Pubkey,
    pub treasury_pern_account: Pubkey,
    pub listing_fee_usd_star: u64,
    pub sale_commission_bps: u16,
    pub is_paused: bool,
    pub bump: u8,
}

// Asset Module State
#[account]
#[derive(Debug)]
pub struct AssetAccount {
    pub creator: Pubkey,
    pub current_owner: Pubkey,
    pub asset_id_hash: [u8; 32],
    pub walrus_main_metadata_cid: String,
    pub ownership_verification_status: OwnershipStatus,
    pub asset_listed_status: AssetListedStatus,
    pub active_auction_key: Option<Pubkey>,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum OwnershipStatus {
    NotSubmitted,
    PendingReview,
    Verified,
    Rejected,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum AssetListedStatus {
    Unlisted,
    AwaitingOwnershipVerification,
    ReadyForAuction,
    InAuction,
    Sold,
}

// Auction Module State
#[account]
#[derive(Debug)]
pub struct AuctionAccount {
    pub asset_key: Pubkey,
    pub seller: Pubkey,
    pub pern_usd_star_mint: Pubkey,
    pub start_price_usd_star: u64,
    pub auction_end_timestamp: i64,
    pub highest_bid_usd_star: u64,
    pub highest_bidder: Option<Pubkey>,
    pub auction_status: AuctionProcessStatus,
    pub escrow_authority_bump: u8,
    pub bump: u8,
}

#[account]
#[derive(Debug)]
pub struct CrossChainAttestation {
    pub user_solana_key: Pubkey,
    pub source_chain_id: u16,
    pub source_asset_hash: [u8; 32],
    pub attested_balance: u64,
    pub attestation_timestamp: i64,
    pub bump: u8,
}

#[account]
#[derive(Debug)]
pub struct WormholeListenerConfig {
    pub wormhole_authorized_relayer: Pubkey,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum AuctionProcessStatus {
    PendingStart,
    Active,
    EndedUnsold,
    EndedSoldPayPending,
    Completed,
}

// Event Structs
#[event]
#[derive(Debug)]
pub struct OwnershipVerificationUpdatedEvent {
    pub asset_key: Pubkey,
    pub status: OwnershipStatus,
    pub notes_hash: Option<[u8; 32]>,
}

#[event]
#[derive(Debug)]
pub struct AssetSold {
    pub asset_key: Pubkey,
    pub new_owner: Pubkey,
    pub walrus_main_metadata_cid: String,
}

#[event]
#[derive(Debug)]
pub struct BidPlacedEvent {
    pub auction_key: Pubkey,
    pub bidder: Pubkey,
    pub amount: u64,
}

#[event]
#[derive(Debug)]
pub struct AuctionEndedWinner {
    pub auction_key: Pubkey,
    pub winner: Pubkey,
    pub winning_bid: u64,
}

#[event]
#[derive(Debug)]
pub struct AuctionEndedNoSale {
    pub auction_key: Pubkey,
}

#[event]
#[derive(Debug)]
pub struct CrossChainBalanceAttestedEvent {
    pub user_solana_key: Pubkey,
    pub source_chain_id: u16,
    pub source_asset_hash: [u8; 32],
    pub attested_balance: u64,
}

// Data Structures for Instruction Parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BalanceAttestationPayload {
    pub solana_target_address: Pubkey,
    pub evm_chain_id: u16,
    // **ENSURE THIS FIELD NAME IS USED CONSISTENTLY**
    pub asset_address_on_evm_as_bytes: Vec<u8>, // Using Vec<u8> for flexibility. If it's always [u8;20], you can use that.
    pub balance: u64,
    pub timestamp: i64,
}
