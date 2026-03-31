//! NFT Presale Time Calibration Module
//!
//! This module handles release time adjustments for NFT nodes that were sold during the presale period,
//! before the contract was fully deployed. To ensure fairness for early NFT node purchasers, this code
//! calibrates user NFT timestamps to align with the actual time they acquired their NFT nodes during presale.
//!
//! This implementation is the result of negotiations with market stakeholders to maintain presale fairness
//! and ensure proper release timing based on the original purchase dates.

use anchor_lang::prelude::*;

/// Presale NFT list (must be sorted by mint address in ascending order for binary search)
/// Format: (nft_mint_str, preset_bound_at_unix_timestamp)
pub const PRESALE_NFTS: &[(&str, i64)] = &[
    ("24UPSULoEaeaFpq9R8JA9pYoJaKSmjT8uJSk2cDgymmp", 1774754694), // 2026-03-29T03:24:54.000Z
    ("2NMnCigDhdaEUPJeN2Ao6SEv57ytDhPf1yww1R5Vc75J", 1773486892), // 2026-03-14T11:14:52.000Z
    ("2VrTfBPM2fANFfVSq9ZytVEMGyRK2ibVqg6jwfsm5wCs", 1774754614), // 2026-03-29T03:23:34.000Z
    ("2jHiwekY16yeGXuxtFnd5LZ5Ef1uFjMj3USadXfG5rum", 1774077296), // 2026-03-21T07:14:56.000Z
    ("2omSo5HJmN4QugZ2dNHu1hYrSdTQuJ1C19R4N4YMXWWm", 1774081162), // 2026-03-21T08:19:22.000Z
    ("2tkRNLqrwP2VQ7eS68J7C5c4puvnptgoUXLWs7bUCad8", 1774754752), // 2026-03-29T03:25:52.000Z
    ("3CH6VMBhnatpLAXzAYPihqj3oHBpf1CggYyjckFEmAGj", 1773723498), // 2026-03-17T04:58:18.000Z
    ("3K4ygAp7i4CnpjvSTQCKUA9Qx5NF4mcvdR6NDGchEZqU", 1774080702), // 2026-03-21T08:11:42.000Z
    ("4hsAJQG7HyVsaRM7YqoDY4C9QG4RSAekXS4Q4LRJvcJ6", 1774084096), // 2026-03-21T09:08:16.000Z
    ("4iTZjZhnySQDsuVsfi6euvzYRFmPtT1Xg5jRZ2xeodvZ", 1774756729), // 2026-03-29T03:58:49.000Z
    ("4wHdEmfLwRoMPCNyX5XMxspYaf8FhRqmcYgnGeZ7QSYr", 1774754735), // 2026-03-29T03:25:35.000Z
    ("5PDweDhJqijParB26uW9QpSzmVmFmC97uCBoyFvbAs3P", 1773140029), // 2026-03-10T10:53:49.000Z
    ("5ULmHbuQeC7VWmxgn5ZR1aMNNucrZrGpQuvynhcySxFS", 1774081141), // 2026-03-21T08:19:01.000Z
    ("5jUdAjMuYvcDWwR9h8P686Qv1aMn3EHzx4uA18JZu2dD", 1774080718), // 2026-03-21T08:11:58.000Z
    ("7kFwxdK5wNP2aMv2EPfDfVU981MNPa7eSXLFZdME9wCj", 1774077365), // 2026-03-21T07:16:05.000Z
    ("7rLEqbuq2iVjFR2kzMnQfi8WtyusPL72dTRahTboqhL5", 1774084101), // 2026-03-21T09:08:21.000Z
    ("95828hcr82GCi8dVKpVqoN9DsHRjmrD3rDusoCZv53ff", 1774077334), // 2026-03-21T07:15:34.000Z
    ("9CKppCRGw3c2BcRoVThh8BDKQfpv7jTa9t7ptyKW58RV", 1774077355), // 2026-03-21T07:15:55.000Z
    ("9SSG5fpWgwLCt8z7dHDXGYBbCLdYzwzoSBEbTXyskWUh", 1773906407), // 2026-03-19T07:46:47.000Z
    ("9Xd6pMBVnthv6kDUWhoSyqFJgkCuhuiP34jNDhgvWucZ", 1773965609), // 2026-03-20T00:13:29.000Z
    ("A2PWzoZm8Q6gS42mSdbhDgNMNBQJhuCdptEgKkpknrBW", 1774757402), // 2026-03-29T04:10:02.000Z
    ("A74kJSn815PgpoKorcfBTuPohdZJCVzBU2ty796zmNnQ", 1773723450), // 2026-03-17T04:57:30.000Z
    ("AH3PkMZkvdmYyD93bRoEKKXGuTztJUk1w3oBxNNsZR3L", 1774754626), // 2026-03-29T03:23:46.000Z
    ("Aaiu4MXD6X8jGuDkEKW3XFQCtjBchk5AQTrFn1C8MxUP", 1773723497), // 2026-03-17T04:58:17.000Z
    ("AdWjqarcSqGz2dVNTfciBUa6cnRpqryEUZd1mwiCwHrd", 1774754765), // 2026-03-29T03:26:05.000Z
    ("Bj7N6D2xdBqLk3fh3uzPuka9dB6cLKAuN2Reb8CHqzBQ", 1774077339), // 2026-03-21T07:15:39.000Z
    ("C1KmRzDLpM9bKj1GbgxZ5CGLciAG3T88X4sqRorEPHLy", 1774341037), // 2026-03-24T08:30:37.000Z
    ("C2UwUvLR8ogxH6oNPn6AYbDUaR5Z7JtvW6fYTsUBDwBc", 1774077370), // 2026-03-21T07:16:10.000Z
    ("CUipGUa9eTWAkw5ig1iSRBHHjXYQubQwabXiVhTYCEN4", 1774077328), // 2026-03-21T07:15:28.000Z
    ("CdCvJNRC19aQAE8pgaeTaCnptnd8wwNJDui79DUfuF6U", 1774754716), // 2026-03-29T03:25:16.000Z
    ("CdEWpofxnkLEgepeqmDbRtVBgB9dVYUJtnbB3hZipPJr", 1773906417), // 2026-03-19T07:46:57.000Z
    ("CfnqHyTLpXLao4msjvQ7AoXpZKp9zWrJ5kNE62o1xZpv", 1774754657), // 2026-03-29T03:24:17.000Z
    ("DPRKiGMcvZHNw2imFez1nDQNVianKtUMut2P2byFMJT6", 1774081136), // 2026-03-21T08:18:56.000Z
    ("DaA93ZhPvhyN4Bk5D6miijE9W7Jw733DN6jBWySaHHPj", 1773723448), // 2026-03-17T04:57:28.000Z
    ("E59Zptp7NyBokrNaagsFSKoCcWQEGamJhHfxexvAFswU", 1774754700), // 2026-03-29T03:25:00.000Z
    ("EjbdRAKnFRfTfiC9QpD74yyx9D9D6EkKjFL2GkD5yAWA", 1773924886), // 2026-03-19T12:54:46.000Z
    ("F8mBv6vcZaAF22JuR9EAXz5T2yJTfHZsEj5ijPE1FueB", 1774754689), // 2026-03-29T03:24:49.000Z
    ("FSEKbdMy76yJYKGywghR1UUZmoo73vvYs957GjsSrjub", 1774080729), // 2026-03-21T08:12:09.000Z
    ("FffrCtucawQDrVdezgGYcnesAYYz7Wzu2vv2ebFnzcSt", 1774756734), // 2026-03-29T03:58:54.000Z
    ("FnZW5rRRphSovJBsJ5c6vX9cP6nfoyoZKnFiKF9bSyMt", 1774077328), // 2026-03-21T07:15:28.000Z
    ("G5wucn9iTQt3xAhPX7kEfaKZmFNSRUBDc7keTRYFnrGE", 1774077323), // 2026-03-21T07:15:23.000Z
    ("G8Dzo3X9dWhfB7ehyhp3uULWHr5YkczkEiTdCqp4sJhA", 1774077301), // 2026-03-21T07:15:01.000Z
    ("GVL2zoMySUJ8StrtqFwGh31bCsnztLMCynar4WwvxTBM", 1773915636), // 2026-03-19T10:20:36.000Z
    ("GkEDafEWRMgX4R3KSzbnNTAMkkdangiHEPrCcP3xtvJc", 1774080697), // 2026-03-21T08:11:37.000Z
    ("HL8oesiad6yskGTESH4UsXfSmv89cou1zhcKBamsrVzk", 1774081120), // 2026-03-21T08:18:40.000Z
    ("HqQs9Mm9DkxaVPJJ5YjYMsYprwb1BTgFjZdt5FcXqDBf", 1774077344), // 2026-03-21T07:15:44.000Z
    ("a33kY8YVv8Ab9nnEPa3pRbJJ19Veo3q6bTzFPY9isCg", 1774084106), // 2026-03-21T09:08:26.000Z
];

/// Query whether an NFT is a presale NFT, and return the preset bound start time if it is
/// PRESALE_NFTS must be sorted by mint address in ascending order
pub fn get_presale_bound_time(nft_mint: &Pubkey) -> Option<i64> {
    let nft_mint_str = nft_mint.to_string();
    PRESALE_NFTS
        .binary_search_by_key(&nft_mint_str.as_str(), |&(mint, _)| mint)
        .ok()
        .map(|idx| PRESALE_NFTS[idx].1)
}
