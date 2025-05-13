use anchor_lang::prelude::*;

#[error_code]
pub enum DubuuMarketplaceError {
    #[msg("Unauthorized access or action")] // Slightly more generic message
    Unauthorized,
    
    #[msg("Invalid Perena USD* mint address provided")] // Slightly more specific
    InvalidPerenaMint,
    
    #[msg("Marketplace operations are currently paused")] // Slightly more descriptive
    MarketplacePaused,
    
    #[msg("Asset ownership verification is required before this action")] // Slightly more context
    OwnershipVerificationRequired,
    
    #[msg("Asset is not in the correct state to be listed for auction")] // More specific than generic "AssetNotReadyForAuction"
    AssetNotReadyForAuction,
    
    #[msg("Asset is already in an active auction")]
    AssetAlreadyInAuction, // Kept as is, clear
    
    #[msg("Auction has already ended")]
    AuctionEnded, // Kept as is, clear
    
    #[msg("Auction has not ended yet")]
    AuctionNotEnded, // Kept as is, clear
    
    #[msg("Bid amount is too low or not greater than the current highest bid")] // More context
    BidTooLow,
    
    #[msg("Auction is not in the expected status for this operation")]
    InvalidAuctionStatus, // Kept as a general fallback
    
    #[msg("Asset is not in the expected status for this operation")]
    InvalidAssetStatus, // Kept as a general fallback

    #[msg("Cross-chain attestation data is invalid or malformed")] // More context
    InvalidAttestationData,
    
    #[msg("Provided string exceeds maximum allowed length")] // More context
    StringTooLong,

    // --- NEWLY ADDED ERROR CODES BASED ON RECOMMENDATIONS ---

    #[msg("Token account owner does not match the expected owner")]
    InvalidTokenAccountOwner,

    #[msg("Provided treasury account does not match the configured treasury account")]
    InvalidTreasuryAccount,

    #[msg("The specified account is not valid for receiving rent (e.g., not the seller)")]
    InvalidRentRecipient,

    #[msg("Timestamp calculation resulted in an overflow or invalid time")]
    TimestampOverflow,

    #[msg("Arithmetic calculation resulted in an overflow or underflow")]
    CalculationOverflow,

    #[msg("Required token account for the previous highest bidder was not provided when necessary")]
    MissingPreviousBidderAccount,

    #[msg("Signer is not the recorded winner of the auction")]
    NotAuctionWinner,

    #[msg("Provided asset account is not the one associated with this auction or operation")]
    InvalidAssetAccount,

    #[msg("Auction is not in an active state for bidding or finalization attempts")]
    AuctionNotInActiveState,

    #[msg("Auction is not in the correct state for settlement (e.g., not EndedSoldPayPending)")]
    AuctionNotInSettlementState,

    #[msg("Current asset status prevents this update or operation")]
    AssetStatusPreventsUpdate,

    #[msg("Auction is not active")]
    AuctionNotActive,

    #[msg("Invalid seller account for rent")]
    InvalidSellerAccountForRent,
}