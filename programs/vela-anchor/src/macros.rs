/// Verify that the 9 storage PDA addresses match those stored in GlobalState.
///
/// Pure key comparison, zero hash computation, extremely low CU cost (~270 CU).
///
/// Usage:
/// ```rust
/// let global_state = &ctx.accounts.global_state;
/// verify_storage_pdas!(ctx, global_state);
/// ```
///
/// `global_state` is a `&GlobalState` reference, passed in from the caller.
#[macro_export]
macro_rules! verify_storage_pdas {
    ($ctx:expr, $global_state:expr) => {{
        use anchor_lang::prelude::require_keys_eq;
        use $crate::errors::ReferralError;

        let storage_refs: [(&anchor_lang::prelude::UncheckedAccount, usize); 9] = [
            (&$ctx.accounts.storage_1, 0),
            (&$ctx.accounts.storage_2, 1),
            (&$ctx.accounts.storage_3, 2),
            (&$ctx.accounts.storage_4, 3),
            (&$ctx.accounts.storage_5, 4),
            (&$ctx.accounts.storage_6, 5),
            (&$ctx.accounts.storage_7, 6),
            (&$ctx.accounts.storage_8, 7),
            (&$ctx.accounts.storage_9, 8),
        ];
        for (acc, i) in storage_refs.iter() {
            require_keys_eq!(
                acc.key(),
                $global_state.storage_pdas[*i],
                ReferralError::InvalidStoragePDA
            );
        }
    }};
}
