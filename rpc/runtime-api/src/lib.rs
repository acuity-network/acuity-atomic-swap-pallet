#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
use sp_std::prelude::Vec;

sp_api::decl_runtime_apis! {
    pub trait AtomicSwapApi<AcuityAssetId, AccountId, Balance, BlockNumber> where
        AcuityAssetId: codec::Codec,
        AccountId: codec::Codec,
		Balance: codec::Codec,
        BlockNumber: codec::Codec,
    {
        fn get_stashes(asset_id: AcuityAssetId, offset: u32, limit: u32) -> sp_std::prelude::Vec<(AccountId, Balance)>;
        fn get_index_blocks(account: AccountId) -> sp_std::prelude::Vec<BlockNumber>;
    }
}
