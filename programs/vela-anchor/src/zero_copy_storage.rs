use anchor_lang::prelude::*;
use crate::structs::ReferralData;
use crate::errors::ReferralError;

/// Account header size (fixed)
/// Layout: 8 (discriminator) + 1 (index) + 4 (count) + 3 (reserved) = 16 bytes
pub const HEADER_SIZE: usize = 16;

/// Single record size (fixed)
pub const RECORD_SIZE: usize = ReferralData::SIZE;

/// Calculate record offset in account
#[inline]
pub fn record_offset(index: u32) -> usize {
    HEADER_SIZE + (index as usize) * RECORD_SIZE
}

/// Zero-copy read single record
///
/// # Parameters
/// - `data`: account raw data
/// - `index`: record index (starting from 0)
///
/// # Returns
/// - `Ok(ReferralData)`: successfully read record
/// - `Err`: index out of bounds or deserialization failed
pub fn read_record(data: &[u8], index: u32) -> Result<ReferralData> {
    let offset = record_offset(index);

    // Boundary check
    require!(
        data.len() >= offset + RECORD_SIZE,
        ReferralError::InvalidIndex
    );

    // Directly deserialize single record from raw bytes (56 bytes, stack-safe)
    let bytes = &data[offset..offset + RECORD_SIZE];
    AnchorDeserialize::deserialize(&mut &bytes[..])
        .map_err(|_| ReferralError::InvalidData.into())
}

/// Zero-copy write single record
///
/// # Parameters
/// - `data`: account raw data (mutable)
/// - `index`: record index (starting from 0)
/// - `record`: record to write
///
/// # Returns
/// - `Ok(())`: successfully written
/// - `Err`: index out of bounds or serialization failed
pub fn write_record(data: &mut [u8], index: u32, record: &ReferralData) -> Result<()> {
    let offset = record_offset(index);

    // Boundary check
    require!(
        data.len() >= offset + RECORD_SIZE,
        ReferralError::InvalidIndex
    );

    // Directly serialize to specified position
    let mut writer = &mut data[offset..offset + RECORD_SIZE];
    record.serialize(&mut writer)
        .map_err(|_| ReferralError::InvalidData.into())
}

/// Zero-copy read count field
///
/// # Parameters
/// - `data`: account raw data
///
/// # Returns
/// - `Ok(u32)`: current record count
/// - `Err`: insufficient data length
pub fn read_count(data: &[u8]) -> Result<u32> {
    require!(data.len() >= 13, ReferralError::InvalidData);
    // count is located at offset 9-12: 8 (discriminator) + 1 (index)
    Ok(u32::from_le_bytes([data[9], data[10], data[11], data[12]]))
}

/// Zero-copy update count field
///
/// # Parameters
/// - `data`: account raw data (mutable)
/// - `count`: new record count
///
/// # Returns
/// - `Ok(())`: successfully updated
/// - `Err`: insufficient data length
pub fn update_count(data: &mut [u8], count: u32) -> Result<()> {
    require!(data.len() >= 13, ReferralError::InvalidData);
    data[9..13].copy_from_slice(&count.to_le_bytes());
    Ok(())
}

/// Zero-copy read index field
///
/// # Parameters
/// - `data`: account raw data
///
/// # Returns
/// - `Ok(u8)`: PDA index (1-9)
/// - `Err`: insufficient data length
pub fn read_index(data: &[u8]) -> Result<u8> {
    require!(data.len() >= 9, ReferralError::InvalidData);
    // index located at offset 8: 8 (discriminator)
    Ok(data[8])
}

/// Zero-copy read wallet address of specified record (read only 32 bytes)
///
/// # Parameters
/// - `data`: account raw data
/// - `index`: record index
///
/// # Returns
/// - `Ok(Pubkey)`: wallet address
/// - `Err`: index out of bounds or invalid data
pub fn read_wallet_at(data: &[u8], index: u32) -> Result<Pubkey> {
    let offset = record_offset(index);
    require!(data.len() >= offset + 32, ReferralError::InvalidIndex);

    let wallet_bytes = &data[offset..offset + 32];
    Pubkey::try_from(wallet_bytes).map_err(|_| ReferralError::InvalidData.into())
}

/// Zero-copy read parent_id of specified record (read only 4 bytes)
///
/// # Parameters
/// - `data`: account raw data
/// - `index`: record index
///
/// # Returns
/// - `Ok(u32)`: parent_id
/// - `Err`: index out of bounds
pub fn read_parent_id_at(data: &[u8], index: u32) -> Result<u32> {
    let offset = record_offset(index) + 32; // after wallet
    require!(data.len() >= offset + 4, ReferralError::InvalidIndex);

    Ok(u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

/// Initialize account header
///
/// # Parameters
/// - `data`: account raw data (mutable)
/// - `index`: PDA index (1-9)
///
/// # Returns
/// - `Ok(())`: successfully initialized
/// - `Err`: insufficient data length
pub fn init_header(data: &mut [u8], index: u8) -> Result<()> {
    require!(data.len() >= HEADER_SIZE, ReferralError::InvalidData);

    // Write index (offset 8)
    data[8] = index;

    // Write count = 0 (offset 9-12)
    data[9..13].copy_from_slice(&0u32.to_le_bytes());

    // Write reserved = [0, 0, 0] (offset 13-15)
    data[13..16].copy_from_slice(&[0u8; 3]);

    Ok(())
}

