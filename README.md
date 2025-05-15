# Dubuu Marketplace MVP WEBSITE https://dubuu.vercel.app/ 
Revolutionizing Automotive Asset Exchange. Securely buy, sell, rent, or leverage your dream car as a tokenized asset on the Solana blockchain.**

Dubuu Marketplace MVP is an advanced decentralized application (dApp) engineered on the Solana blockchain, leveraging the Anchor framework. It establishes a transparent and efficient ecosystem for the tokenization, management, and exchange of high-value real-world assets (RWAs), with an initial focus on automotive assets. The platform integrates robust features for asset registration, cryptographically verifiable ownership, dynamic auction mechanisms, and secure cross-chain interactions.

## Table of Contents

- [Vision: The Dubuu Advantage](#vision-the-dubuu-advantage)
- [Key Technical Features](#key-technical-features)
- [Architectural Highlights](#architectural-highlights)
- [Core Smart Contract Modules](#core-smart-contract-modules)
  - [Configuration Module](#configuration-module)
  - [Asset Module](#asset-module)
  - [Auction Module](#auction-module)
  - [Cross-Chain Module (Wormhole Integration)](#cross-chain-module-wormhole-integration)
- [Key Technology Integrations](#key-technology-integrations)
  - [Perena USD* (SPL Token)](#perena-usd-spl-token)
  - [Walrus & IPFS (Decentralized Metadata)](#walrus--ipfs-decentralized-metadata)
  - [Wormhole (Cross-Chain Communication)](#wormhole-cross-chain-communication)
- [Technical Stack](#technical-stack)
- [Detailed Project Structure](#detailed-project-structure)
- [Development & Deployment Lifecycle](#development--deployment-lifecycle)
  - [Prerequisites](#prerequisites)
  - [Local Setup & Installation](#local-setup--installation)
  - [Building the Program](#building-the-program)
  - [Testing Suite](#testing-suite)
  - [Deployment to Solana Clusters](#deployment-to-solana-clusters)
- [Operational Flows (Usage)](#operational-flows-usage)
- [Contribution Guidelines](#contribution-guidelines)
- [License](#license)

## Vision: The Dubuu Advantage

The traditional automotive market is often encumbered by intermediaries, opaque processes, and geographical limitations. Dubuu challenges this paradigm by:

* **Democratizing Access:** Enabling a global pool of participants to engage with automotive assets, from direct ownership to fractional investment and lending.
* **Enhancing Transparency:** Leveraging Solana's immutable ledger for all transactions, ownership records, and auction histories.
* **Increasing Liquidity:** Transforming illiquid automotive assets into tradable on-chain tokens, opening new avenues for financing and investment.
* **Reducing Friction:** Streamlining processes like ownership transfer and payment settlement through efficient smart contract execution.
* **Fostering Trust:** Implementing verifiable ownership processes and secure, decentralized auction mechanisms.

Dubuu aims to be the premier platform for tokenized automotive assets, offering unparalleled security, efficiency, and global reach.

## Key Technical Features

* **Decentralized Asset Registry (PDAs):** Unique on-chain representation of each car asset using Program Derived Addresses for deterministic and verifiable storage.
* **Verifiable Ownership Protocol:** Multi-stage process for asset ownership verification, recorded immutably.
* **Dynamic Auction System:** Configurable auction parameters (start price, duration) with on-chain bid tracking and automated settlement logic.
* **SPL Token Integration (Perena USD\*):** Seamless use of Perena USD\* for all platform-native value transfers (fees, bids, commissions) via CPIs to the SPL Token Program.
* **Cross-Chain Attestation (Wormhole):** Capability to ingest and verify balance or asset data from external blockchains through Wormhole's VAA mechanism.
* **Role-Based Access Control:** Differentiated permissions for administrators, asset owners, and general users.
* **Event-Driven Architecture:** Emission of granular on-chain events for critical state changes, enabling robust off-chain indexing and real-time UI updates.
* **Decentralized Metadata Linking (IPFS/Walrus):** Asset metadata (specifications, history, images) stored on IPFS, with CIDs linked on-chain, potentially adhering to Walrus conventions for discoverability.

## Architectural Highlights

* **PDA-Centric Design:** Extensive use of Program Derived Addresses for creating unique, deterministic account keys for all major state objects (`MarketplaceConfig`, `AssetAccount`, `AuctionAccount`, etc.), enhancing security and composability.
* **Modular Smart Contracts:** Logic is segregated into distinct Rust modules (`config_module`, `asset_module`, `auction_module`) for clarity, maintainability, and testability.
* **CPI (Cross-Program Invocation):** Secure interaction with the SPL Token program for all token transfers, ensuring adherence to Solana's token standards.
* **Custom Error Handling:** Comprehensive `ErrorCode` enum for clear and debuggable on-chain error reporting.
* **State Compression (Future Consideration):** While not explicitly in the MVP state, the architecture is amenable to future integration with Solana's state compression for managing large numbers of asset accounts cost-effectively.

## Core Smart Contract Modules

The smart contract functionality is logically divided as follows:

### Configuration Module

* **Purpose:** Manages global, admin-controlled marketplace parameters.
* **Primary State Account:** `MarketplaceConfig` (Singleton PDA).
    * `admin`: `Pubkey` with authority over critical configuration changes.
    * `pern_usd_star_mint`: `Pubkey` of the Perena USD\* SPL Token.
    * `treasury_pern_account`: `Pubkey` of the marketplace's Associated Token Account (ATA) for Perena USD\*.
    * `listing_fee_usd_star`: `u64` fee for listing an asset.
    * `sale_commission_bps`: `u16` commission (basis points) on sales.
    * `is_paused`: `bool` to halt specific marketplace functions.
* **Key Instructions:**
    * `initialize_config`: Deploys and initializes the `MarketplaceConfig` PDA.
    * `update_config`: Modifies fields in `MarketplaceConfig`, restricted to the `admin`.

### Asset Module

* **Purpose:** Governs asset registration, metadata management, and ownership verification.
* **Primary State Account:** `AssetAccount` (PDA, typically seeded by a unique asset identifier).
    * `creator`: `Pubkey` of the initial asset registrant.
    * `current_owner`: `Pubkey` of the verified beneficial owner.
    * `asset_id_hash`: `[u8; 32]` unique hash derived from asset specifics (e.g., VIN hash).
    * `walrus_main_metadata_cid`: `String` (IPFS CID) for detailed off-chain metadata.
    * `ownership_verification_status`: `OwnershipStatus` enum.
    * `asset_listed_status`: `AssetListedStatus` enum.
    * `active_auction_key`: `Option<Pubkey>` linking to an active `AuctionAccount`.
* **Key Instructions:**
    * `register_asset_and_submit_docs_ref`: Creates an `AssetAccount` PDA, initializing it with metadata and setting status to `PendingReview` or `NotSubmitted`.
    * `admin_update_ownership_verification`: Admin-only instruction to transition `ownership_verification_status`.
    * `update_asset_walrus_cid`: Allows owner/admin to update the metadata link.
* **Emitted Events:** `OwnershipVerificationUpdatedEvent`, `AssetSoldEvent`.

### Auction Module

* **Purpose:** Manages the on-chain auction lifecycle for verified assets.
* **Primary State Account:** `AuctionAccount` (PDA, typically seeded by the `AssetAccount` key and a nonce/timestamp).
    * `asset_key`: `Pubkey` of the `AssetAccount` being auctioned.
    * `seller`: `Pubkey` of the asset owner at the time of listing.
    * `pern_usd_star_mint`: `Pubkey` of Perena USD\*, ensuring bids are in the correct currency.
    * `start_price_usd_star`, `auction_end_timestamp`, `highest_bid_usd_star`, `highest_bidder`.
    * `auction_status`: `AuctionProcessStatus` enum.
* **Key Instructions:**
    * `list_asset_for_auction`: Creates an `AuctionAccount`, updates `AssetAccount` status. Requires listing fee payment.
    * `place_bid`: Allows users to submit bids. Involves transferring bid amount (Perena USD\*) to an escrow (PDA or temporary token account) or handling refunds for outbid users.
    * `finalize_auction`: Admin or time-triggered instruction to end the auction, determining winner/no-sale.
    * `settle_auction_and_transfer`: Transfers funds to seller (less commission to treasury) and updates `AssetAccount` owner to the winner.
* **Emitted Events:** `BidPlacedEvent`, `AuctionEndedWinnerEvent`, `AuctionEndedNoSaleEvent`.

### Cross-Chain Module (Wormhole Integration)

* **Purpose:** Facilitates the ingestion and verification of attestations from external blockchains.
* **Primary State Accounts:**
    * `CrossChainAttestation` (PDA): Stores verified data from a Wormhole VAA (Verifiable Action Approval).
    * `WormholeListenerConfig` (PDA): Configures the authorized Wormhole relayer address.
* **Key Instructions:**
    * `initialize_wormhole_listener`: Sets the trusted relayer.
    * `process_wormhole_balance_attestation`: Consumes a Wormhole VAA (provided by a relayer) to record an attested balance for a user from an EVM chain, storing it in a `CrossChainAttestation` account. The `BalanceAttestationPayload` struct defines the expected data structure from the VAA.
* **Emitted Events:** `CrossChainBalanceAttestedEvent`.

## Key Technology Integrations

### Perena USD* (SPL Token)
* **Technical Role:** Serves as the standardized SPL Token for all value-based interactions.
* **Implementation:** Transactions involving Perena USD\* (fees, bids, payouts) are executed via CPIs to the official SPL Token Program (`TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`). Associated Token Accounts (ATAs) are used for user and treasury balances.
* **Reference:** [Perena Product Documentation](https://perena.notion.site/Product-Documentation-15fa37a29ca48060afd9cabb21b44d5c)

### Walrus & IPFS (Decentralized Metadata)
* **Technical Role:** IPFS provides decentralized storage for rich asset metadata (images, documents, specifications). The `walrus_main_metadata_cid` stored on-chain is an IPFS CID.
* **Implementation:** While "Walrus CID" is mentioned, Walrus.xyz is primarily an explorer. The term likely implies that the metadata at the IPFS CID follows a schema that Walrus or similar explorers can readily parse and display. The smart contract only stores and manages the CID link.
* **Reference:** [IPFS Docs](https://docs.ipfs.tech/), [Walrus.xyz](https://www.walrus.xyz/)

### Wormhole (Cross-Chain Communication)
* **Technical Role:** Acts as a generic message-passing bridge to receive Verifiable Action Approvals (VAAs) from other supported blockchains (e.g., EVM chains).
* **Implementation:** The `process_wormhole_balance_attestation` instruction is designed to be called by an authorized relayer that submits a parsed and verified VAA. The contract then processes the payload within this VAA.
* **Reference:** [Wormhole Documentation](https://docs.wormhole.com/)

## Technical Stack

* **Blockchain Platform:** Solana
* **Smart Contract Language & Framework:** Rust with Anchor
* **Standard Currency:** Perena USD\* (SPL Token)
* **Cross-Chain Messaging:** Wormhole Network
* **Decentralized Metadata Storage:** InterPlanetary File System (IPFS)

## Development & Deployment Lifecycle

### Prerequisites

* Rust (version compatible with your chosen Anchor version, e.g., as per Anchor docs)
* Solana Tool Suite (latest stable recommended): [Install Guide](https://docs.solana.com/cli/install)
* Anchor Framework (AVM recommended for version management): [Install Guide](https://www.anchor-lang.com/docs/installation)
* Node.js (LTS) & Yarn (or npm)

### Local Setup & Installation

1.  **Clone Repository:**
    ```bash
    git clone [https://github.com/Lappsnet/dubuu.git](https://github.com/Lappsnet/dubuu.git)
    cd dubuu
    ```
2.  **Install JS Dependencies:**
    ```bash
    yarn install # or npm install
    ```
3.  **Configure Solana CLI for Localnet:**
    ```bash
    solana config set --url localhost
    solana config set --keypair ~/.config/solana/id.json # Default local wallet
    solana airdrop 2 # Fund your local wallet
    ```

### Building the Program

Compile the Anchor smart contract to BPF bytecode and generate the IDL:
```bash
anchor build
```

### Testing Suite
Execute integration tests against a local validator instance:

Start local validator in a separate terminal (if not already running)
```bash
anchor localnet
```

Run tests
```bash
anchor test
```

Or, if localnet is managed separately:
```bash
anchor test --skip-local-validator
```

### Operational Flows (Usage)
The Dubuu Marketplace dApp enables distinct operational pathways tailored to different user roles, ensuring a structured and secure interaction with the platform's functionalities. These flows are orchestrated by invoking specific instructions on the smart contract.

Marketplace Administrator Flow:

## User Roles and Flows

### 1. Marketplace Administrator Flow

The administrator is responsible for the initial setup, ongoing management, and integrity oversight of the marketplace.

**a. Initialization:**
   - **Action:** Performs the initial setup of the marketplace.
   - **Instruction:** `initialize_config`
   - **Details:** This crucial step establishes the `MarketplaceConfig` Program Derived Address (PDA). It defines foundational parameters such as:
     - The designated Perena USD* mint.
     - The treasury account for fee collection.
     - Initial listing fees.
     - Sales commission rates.
     - The administrative public key.

**b. Configuration Management:**
   - **Action:** Modifies operational parameters after initialization.
   - **Instruction:** `update_config`
   - **Details:** Allows for dynamic adjustment of:
     - Fees and commission rates.
     - The treasury account.
     - The master admin key.
     - The ability to pause or unpause core marketplace activities (`is_paused` flag), providing essential governance and risk management.

**c. Asset Verification Oversight:**
   - **Action:** Reviews submitted ownership proofs and updates an asset's `OwnershipStatus` on-chain.
   - **Instruction:** `admin_update_ownership_verification`
   - **Details:** The admin reviews off-chain managed ownership proofs (referenced on-chain) and updates the asset's status (e.g., from `PendingReview` to `Verified` or `Rejected`). This curates the quality and legitimacy of assets available for auction.

**d. Operational Control:**
   - **Mechanism:** The `is_paused` flag within `MarketplaceConfig`.
   - **Action:** Allows the admin to temporarily halt new listings or auctions, typically during maintenance or critical updates.

---

### 2. Asset Owner/Seller Flow

This flow outlines the journey of an asset owner looking to sell their automotive asset on the marketplace.

**a. Asset Registration:**
   - **Action:** Registers an automotive asset with the marketplace.
   - **Instruction:** `register_asset_and_submit_docs_ref`
   - **Details:**
     - Creates an `AssetAccount` PDA for the asset.
     - Links the asset to the owner's Solana identity.
     - Associates an IPFS Content Identifier (CID) (`walrus_main_metadata_cid`) which points to detailed off-chain metadata and documentation.
     - The initial `OwnershipStatus` is typically set to `NotSubmitted` or `PendingReview`.

**b. Listing for Auction:**
   - **Prerequisite:** The asset's `OwnershipStatus` must be `Verified` by the administrator.
   - **Action:** Lists the verified asset for sale via auction.
   - **Instruction:** `list_asset_for_auction`
   - **Details:**
     - Creates an `AuctionAccount` PDA.
     - Sets the auction's starting price and duration.
     - Updates the `AssetAccount`'s `asset_listed_status` to `InAuction`.
     - A listing fee, payable in Perena USD*, is typically required.

**c. Receiving Sale Proceeds:**
   - **Prerequisite:** The auction concludes successfully with a winning bid, and the `settle_auction_and_transfer` instruction has been executed.
   - **Action:** The seller receives the final sale price in Perena USD* to their designated account.
   - **Details:** The received amount is the final sale price minus the marketplace commission.

---

### 3. Buyer/Bidder Flow

This flow describes how interested buyers can participate in auctions and acquire assets.

**a. Placing Bids:**
   - **Action:** Participates in active auctions by submitting bids.
   - **Instruction:** `place_bid`
   - **Details:**
     - The bidder specifies their bid amount in Perena USD*.
     - The bid amount is typically transferred to an escrow mechanism managed by the smart contract.
     - If a bidder is outbid, their previous bids might be refunded.

**b. Settlement & Ownership Claim:**
   - **Prerequisite:** The bidder has the `highest_bidder` status when the auction status is `EndedSoldPayPending`.
   - **Action:** Finalizes the payment and claims ownership of the asset.
   - **Instruction:** `settle_auction_and_transfer` (can be executed by the winner or an authorized party/crank).
   - **Details:**
     - Finalizes the payment from the winner's account (or confirms payment from escrow).
     - Updates the `current_owner` field in the corresponding `AssetAccount` to the winner's public key, effectively transferring on-chain ownership.

---

### 4. Wormhole Relayer Flow (Authorized Personnel)

This flow involves authorized personnel who facilitate cross-chain interactions using Wormhole.

**a. Initialization (Setup):**
   - **Action:** Configures the Wormhole relayer listener on Solana.
   - **Instruction:** `initialize_wormhole_listener`

**b. Attestation Submission:**
   - **Action:** Monitors for specific cross-chain messages (Verified Accion Attestations - VAAs), parses them, and submits validated information to the Solana smart contract.
   - **Instruction:** `process_wormhole_balance_attestation`
   - **Details:**
     - Upon receiving a relevant VAA (e.g., attesting to a user's asset balance on an EVM chain), the relayer submits the validated `BalanceAttestationPayload`.
     - This creates or updates a `CrossChainAttestation` account on Solana.
     - The attested off-chain information becomes available to the Dubuu marketplace logic.

---

## Important Considerations

- **Atomicity:** The operational flows are designed to be atomic where necessary to ensure consistency and prevent partial state changes.
- **Event Emission:** The smart contract emits events to allow off-chain services and User Interfaces (UIs) to track marketplace activity and respond accordingly.

*Note: "Perena USD*" refers to a specific token used within the Dubuu Marketplace ecosystem.*

