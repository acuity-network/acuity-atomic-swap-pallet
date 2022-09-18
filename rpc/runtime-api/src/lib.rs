#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
use sp_std::prelude::Vec;

sp_api::decl_runtime_apis! {
    pub trait AtomicSwapApi<AccountId, BlockNumber> where
        AccountId: codec::Codec,
        BlockNumber: codec::Codec,
    {
        fn get_index_blocks(account: AccountId) -> sp_std::prelude::Vec<BlockNumber>;
    }
}
