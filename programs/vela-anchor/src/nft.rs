use anchor_lang::prelude::*;
use std::str::FromStr;
use anchor_spl::{
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3, verify_sized_collection_item,
        mpl_token_metadata::types::{CollectionDetails, Creator, DataV2},
        CreateMasterEditionV3, CreateMetadataAccountsV3, VerifySizedCollectionItem,
    },
    token::{mint_to, MintTo},
};

use crate::{
    contexts::*,
    structs::*,
    events::*,
    errors::NodeError,
    constants,
};

// ============================================================================
// Diamond Collection (Diamond Node)
// ============================================================================

/// Create Diamond Node Collection
pub fn create_diamond_collection(
    ctx: Context<CreateDiamondCollection>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {


    // Permission check: only admins can create
    let admin_pubkey = Pubkey::from_str(constants::NFT_AUTHORITY_ADDRESS)
        .map_err(|_| NodeError::UnauthorizedAdmin)?;
    require!(
        ctx.accounts.payer.key() == admin_pubkey,
        NodeError::UnauthorizedAdmin
    );

    // Initialize Diamond Collection state account
    let collection_state = &mut ctx.accounts.diamond_collection_state;
    collection_state.authority = ctx.accounts.payer.key();
    collection_state.collection_mint = ctx.accounts.collection_mint.key();
    collection_state.minted_count = 0;
    collection_state.max_supply = constants::DIAMOND_NFT_MAX_SUPPLY;
    collection_state.boost_percentage = constants::DIAMOND_BOOST_PERCENTAGE;
    collection_state.bump = ctx.bumps.diamond_collection_state;


    // Mint 1 token to collection mint
    mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.collection_mint.to_account_info(),
                to: ctx.accounts.collection_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        1,
    )?;

    // Create metadata account (use payer as update_authority)
    let creator = vec![Creator {
        address: ctx.accounts.payer.key(),
        verified: true, // Set to true (payer auto-signs)
        share: 100,
    }];

    create_metadata_accounts_v3(
        CpiContext::new(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.collection_metadata.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                mint_authority: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.payer.to_account_info(), // Use payer
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
        DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: Some(creator),
            collection: None,
            uses: None,
        },
        true,
        true,
        Some(CollectionDetails::V1 { size: 0 }),
    )?;

    // Create master edition (make it an NFT and non-mintable)
    create_master_edition_v3(
        CpiContext::new(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.collection_master_edition.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                update_authority: ctx.accounts.payer.to_account_info(),
                mint_authority: ctx.accounts.payer.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                metadata: ctx.accounts.collection_metadata.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
        Some(0), // max_supply = 0 means non-mintable
    )?;


    Ok(())
}

/// Mint Diamond Node NFT (single mint)
pub fn mint_diamond_nft(
    ctx: Context<MintDiamondNFT>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {


    let collection_state = &mut ctx.accounts.diamond_collection_state;

    // Check 1: verify caller is authorized primary address
    require!(
        ctx.accounts.payer.key() == collection_state.authority,
        NodeError::UnauthorizedMinter
    );

    // Check 2: verify if max supply is reached
    require!(
        collection_state.minted_count < collection_state.max_supply,
        NodeError::MaxSupplyReached
    );

    // Check 3: verify collection_mint matches state
    require!(
        ctx.accounts.collection_mint.key() == collection_state.collection_mint,
        NodeError::InvalidCollectionMint
    );



    // Mint 1 token
    mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.nft_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        1,
    )?;

    // Create metadata account
    let creator = vec![Creator {
        address: ctx.accounts.payer.key(),
        verified: true,
        share: 100,
    }];

    create_metadata_accounts_v3(
        CpiContext::new(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.nft_metadata.to_account_info(),
                mint: ctx.accounts.nft_mint.to_account_info(),
                mint_authority: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.payer.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
        DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: Some(creator),
            collection: Some(anchor_spl::metadata::mpl_token_metadata::types::Collection {
                verified: false,
                key: ctx.accounts.collection_mint.key(),
            }),
            uses: None,
        },
        true,
        true,
        None,
    )?;

    // Create master edition (make it an NFT and non-mintable)
    create_master_edition_v3(
        CpiContext::new(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.nft_master_edition.to_account_info(),
                mint: ctx.accounts.nft_mint.to_account_info(),
                update_authority: ctx.accounts.payer.to_account_info(),
                mint_authority: ctx.accounts.payer.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                metadata: ctx.accounts.nft_metadata.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
        Some(0), // max_supply = 0 means non-mintable
    )?;

    // Increase minted count
    collection_state.minted_count += 1;

    // Verify Collection (set verified = true)
    verify_sized_collection_item(
        CpiContext::new(
            ctx.accounts.metadata_program.to_account_info(),
            VerifySizedCollectionItem {
                payer: ctx.accounts.payer.to_account_info(),
                metadata: ctx.accounts.nft_metadata.to_account_info(),
                collection_authority: ctx.accounts.payer.to_account_info(),
                collection_mint: ctx.accounts.collection_mint.to_account_info(),
                collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
                collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
            },
        ),
        None, // collection_authority_record
    )?;

    Ok(())
}

/// Verify user owns Diamond Node NFT
pub fn verify_diamond_ownership(
    ctx: Context<VerifyDiamondOwnership>,
) -> Result<DiamondOwnershipResult> {

    let token_account = &ctx.accounts.user_token_account;
    let nft_mint = &ctx.accounts.nft_mint;

    // Deserialize metadata account
    let metadata_account_data = &ctx.accounts.nft_metadata.data.borrow();
    let metadata = anchor_spl::metadata::MetadataAccount::try_deserialize(&mut &metadata_account_data[..])?;

    // Verify 1: Token Account owner
    let is_owner = token_account.owner == ctx.accounts.user.key();


    // Verify 2: Mint address match
    let is_correct_mint = token_account.mint == nft_mint.key();


    // Verify 3: Balance check
    let balance = token_account.amount;


    // Verify 4: NFT validity (supply=1, decimals=0)
    let is_valid_nft = nft_mint.supply == 1 && nft_mint.decimals == 0;


    // Verify 5: Master Edition exists
    let master_edition_info = &ctx.accounts.nft_master_edition;
    let has_master_edition = master_edition_info.data_is_empty() == false;


    // Verify 6: Collection match and verified
    let collection_verified = if let Some(collection) = &metadata.collection {
        collection.key == ctx.accounts.collection_mint.key()
            && collection.verified  // Must check verified field simultaneously
    } else {
        false
    };

    // Comprehensive judgment
    let owns_nft = is_owner
        && is_correct_mint
        && balance >= 1
        && is_valid_nft
        && has_master_edition
        && collection_verified;



    // Emit event
    emit!(DiamondVerificationEvent {
        user: ctx.accounts.user.key(),
        nft_mint: nft_mint.key(),
        collection_mint: ctx.accounts.collection_mint.key(),
        owns_nft,
        balance,
    });

    // Return verification result
    Ok(DiamondOwnershipResult {
        owns_nft,
        balance,
        nft_mint: nft_mint.key(),
        collection_mint: ctx.accounts.collection_mint.key(),
    })
}

// ============================================================================
// Gold Collection (Gold Node)
// ============================================================================

/// Create Gold Node Collection
pub fn create_gold_collection(
    ctx: Context<CreateGoldCollection>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {


    // Permission check: only admins can create
    let admin_pubkey = Pubkey::from_str(constants::NFT_AUTHORITY_ADDRESS)
        .map_err(|_| NodeError::UnauthorizedAdmin)?;
    require!(
        ctx.accounts.payer.key() == admin_pubkey,
        NodeError::UnauthorizedAdmin
    );

    // Initialize Gold Collection state account
    let collection_state = &mut ctx.accounts.gold_collection_state;
    collection_state.authority = ctx.accounts.payer.key();
    collection_state.collection_mint = ctx.accounts.collection_mint.key();
    collection_state.minted_count = 0;
    collection_state.max_supply = constants::GOLD_NFT_MAX_SUPPLY;
    collection_state.boost_percentage = constants::GOLD_BOOST_PERCENTAGE;
    collection_state.bump = ctx.bumps.gold_collection_state;


    // Mint 1 token to collection mint
    mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.collection_mint.to_account_info(),
                to: ctx.accounts.collection_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        1,
    )?;

    // Create metadata account (use payer as update_authority)
    let creator = vec![Creator {
        address: ctx.accounts.payer.key(),
        verified: true, // Set to true (payer auto-signs)
        share: 100,
    }];

    create_metadata_accounts_v3(
        CpiContext::new(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.collection_metadata.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                mint_authority: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.payer.to_account_info(), // Use payer
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
        DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: Some(creator),
            collection: None,
            uses: None,
        },
        true,
        true,
        Some(CollectionDetails::V1 { size: 0 }),
    )?;

    // Create master edition (make it an NFT and non-mintable)
    create_master_edition_v3(
        CpiContext::new(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.collection_master_edition.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                update_authority: ctx.accounts.payer.to_account_info(),
                mint_authority: ctx.accounts.payer.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                metadata: ctx.accounts.collection_metadata.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
        Some(0), // max_supply = 0 means non-mintable
    )?;


    Ok(())
}

/// Mint Gold Node NFT (single mint)
pub fn mint_gold_nft(
    ctx: Context<MintGoldNFT>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {


    let collection_state = &mut ctx.accounts.gold_collection_state;

    // Check 1: verify caller is authorized primary address
    require!(
        ctx.accounts.payer.key() == collection_state.authority,
        NodeError::UnauthorizedMinter
    );

    // Check 2: verify if max supply is reached
    require!(
        collection_state.minted_count < collection_state.max_supply,
        NodeError::MaxSupplyReached
    );

    // Check 3: verify collection_mint matches state
    require!(
        ctx.accounts.collection_mint.key() == collection_state.collection_mint,
        NodeError::InvalidCollectionMint
    );



    // Mint 1 token
    mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.nft_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        1,
    )?;

    // Create metadata account
    let creator = vec![Creator {
        address: ctx.accounts.payer.key(),
        verified: true,
        share: 100,
    }];

    create_metadata_accounts_v3(
        CpiContext::new(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.nft_metadata.to_account_info(),
                mint: ctx.accounts.nft_mint.to_account_info(),
                mint_authority: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.payer.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
        DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: Some(creator),
            collection: Some(anchor_spl::metadata::mpl_token_metadata::types::Collection {
                verified: false,
                key: ctx.accounts.collection_mint.key(),
            }),
            uses: None,
        },
        true,
        true,
        None,
    )?;

    // Create master edition (make it an NFT and non-mintable)
    create_master_edition_v3(
        CpiContext::new(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.nft_master_edition.to_account_info(),
                mint: ctx.accounts.nft_mint.to_account_info(),
                update_authority: ctx.accounts.payer.to_account_info(),
                mint_authority: ctx.accounts.payer.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                metadata: ctx.accounts.nft_metadata.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
        Some(0), // max_supply = 0 means non-mintable
    )?;

    // Increase minted count
    collection_state.minted_count += 1;

    // Verify Collection (set verified = true)
    verify_sized_collection_item(
        CpiContext::new(
            ctx.accounts.metadata_program.to_account_info(),
            VerifySizedCollectionItem {
                payer: ctx.accounts.payer.to_account_info(),
                metadata: ctx.accounts.nft_metadata.to_account_info(),
                collection_authority: ctx.accounts.payer.to_account_info(),
                collection_mint: ctx.accounts.collection_mint.to_account_info(),
                collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
                collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
            },
        ),
        None, // collection_authority_record
    )?;

    Ok(())
}

/// Verify user owns Gold Node NFT
pub fn verify_gold_ownership(
    ctx: Context<VerifyGoldOwnership>,
) -> Result<GoldOwnershipResult> {

    let token_account = &ctx.accounts.user_token_account;
    let nft_mint = &ctx.accounts.nft_mint;

    // Deserialize metadata account
    let metadata_account_data = &ctx.accounts.nft_metadata.data.borrow();
    let metadata = anchor_spl::metadata::MetadataAccount::try_deserialize(&mut &metadata_account_data[..])?;

    // Verify 1: Token Account owner
    let is_owner = token_account.owner == ctx.accounts.user.key();


    // Verify 2: Mint address match
    let is_correct_mint = token_account.mint == nft_mint.key();


    // Verify 3: Balance check
    let balance = token_account.amount;


    // Verify 4: NFT validity (supply=1, decimals=0)
    let is_valid_nft = nft_mint.supply == 1 && nft_mint.decimals == 0;


    // Verify 5: Master Edition exists
    let master_edition_info = &ctx.accounts.nft_master_edition;
    let has_master_edition = master_edition_info.data_is_empty() == false;


    // Verify 6: Collection match and verified
    let collection_verified = if let Some(collection) = &metadata.collection {
        collection.key == ctx.accounts.collection_mint.key()
            && collection.verified  // Must check verified field simultaneously
    } else {
        false
    };


    // Comprehensive judgment
    let owns_nft = is_owner
        && is_correct_mint
        && balance >= 1
        && is_valid_nft
        && has_master_edition
        && collection_verified;



    // Emit event
    emit!(GoldVerificationEvent {
        user: ctx.accounts.user.key(),
        nft_mint: nft_mint.key(),
        collection_mint: ctx.accounts.collection_mint.key(),
        owns_nft,
        balance,
    });

    // Return verification result
    Ok(GoldOwnershipResult {
        owns_nft,
        balance,
        nft_mint: nft_mint.key(),
        collection_mint: ctx.accounts.collection_mint.key(),
    })
}
