#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_runtime::{traits::AccountIdConversion, RuntimeDebug};
use frame_support::{
    pallet_prelude::MaxEncodedLen,
    traits::{ExistenceRequirement::AllowDeath, Currency},
    PalletId,
};
use scale_info::TypeInfo;
use sp_io::hashing::{blake2_256, keccak_256};

pub use pallet::*;

use sp_std::{
	convert::TryInto,
};


use sp_runtime::{
	traits::{
		Zero
	},
};


use codec::{Encode, Decode};
#[cfg(feature = "serde_derive")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Serialization shim for arbitrary arrays that is consistent with `polkadot-js`'s implementation.
///
/// `polkadot-js` sends us a `0x01020304`, but the default rust implementation for arrays expects a
/// `[0x01, 0x02, 0x03, 0x04]`. Here, we use a similar serialization as substrate uses for `vec`,
/// but we transform it to an array before returning.
#[cfg(feature = "serde_derive")]
pub mod serialize_array {
	use impl_serde::serialize::{deserialize_check_len, ExpectedLen};
	use serde::Deserializer;

	// default serialize is fine
	pub use impl_serde::serialize::serialize;

	pub use deserialize_array as deserialize;

	pub fn deserialize_array<'de, D, const T: usize>(deserializer: D) -> Result<[u8; T], D::Error>
	where
		D: Deserializer<'de>,
	{
		// All hail the stable const generics!
		let mut arr = [0u8; T];
		deserialize_check_len(deserializer, ExpectedLen::Exact(&mut arr[..]))?;

		Ok(arr)
	}
}

/// An Asset Id (i.e. 8 bytes).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AcuityAssetId(
    #[cfg_attr(feature = "std", serde(with = "serialize_array"))]
    [u8; 32]
);

/// A lock ID (i.e. 32 bytes).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AcuityLockId([u8; 32]);

/// A hashed secret (i.e. 32 bytes).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AcuityHashedSecret([u8; 32]);

/// A secret (i.e. 32 bytes).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AcuitySecret([u8; 32]);

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
    pub trait Config: pallet_timestamp::Config + frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// PalletId for the crowdloan pallet. An appropriate value could be ```PalletId(*b"py/cfund")```
		#[pallet::constant]
		type PalletId: Get<PalletId>;

        /// The currency type that the charity deals in
        type Currency: Currency<Self::AccountId>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

        #[pallet::weight(50_000_000)]
		pub fn lock_buy(origin: OriginFor<T>, recipient: T::AccountId, hashed_secret: AcuityHashedSecret, timeout: T::Moment, value: BalanceOf<T>, sell_asset_id: AcuityAssetId, sell_price: u128) -> DispatchResultWithPostInfo {
            let buyer = ensure_signed(origin)?;
            // Ensure value is nonzero.
            frame_support::ensure!(!value.is_zero(), Error::<T>::ZeroValue);
            // Calculate lock_id.
            let lock_id = Self::get_lock_id(buyer.clone(), recipient.clone(), hashed_secret, timeout);
            // Ensure lock_id is not already in use.
            ensure!(!LockIdValue::<T>::contains_key(lock_id), Error::<T>::LockAlreadyExists);
            // Move the value from the sender to the pallet.
            T::Currency::transfer(&buyer, &Self::fund_account_id(), value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Move value into buy lock.
            <LockIdValue<T>>::insert(lock_id, value);
            // Index event.
            Self::index_account(buyer.clone());
            Self::index_account(recipient.clone());
            // Log info.
            Self::deposit_event(Event::BuyLock(buyer, recipient, hashed_secret, timeout, value, lock_id, sell_asset_id, sell_price));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub fn lock_sell(origin: OriginFor<T>, recipient: T::AccountId, hashed_secret: AcuityHashedSecret, timeout: T::Moment, value: BalanceOf<T>, buy_asset_id: AcuityAssetId, buy_lock_id: AcuityLockId) -> DispatchResultWithPostInfo {
            let seller = ensure_signed(origin)?;
            // Ensure value is nonzero.
            frame_support::ensure!(!value.is_zero(), Error::<T>::ZeroValue);
            // Calculate lock_id.
            let lock_id = Self::get_lock_id(seller.clone(), recipient.clone(), hashed_secret, timeout);
            // Ensure lock_id is not already in use.
            ensure!(!LockIdValue::<T>::contains_key(lock_id), Error::<T>::LockAlreadyExists);
            // Move the value from the sender to the pallet.
            T::Currency::transfer(&seller, &Self::fund_account_id(), value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Move value into sell lock.
            <LockIdValue<T>>::insert(lock_id, value);
            // Index event.
            Self::index_account(seller.clone());
            Self::index_account(recipient.clone());
            // Log info.
            Self::deposit_event(Event::SellLock(seller, recipient, hashed_secret, timeout, value, lock_id, buy_asset_id, buy_lock_id));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub fn decline_by_recipient(origin: OriginFor<T>, sender: T::AccountId, hashed_secret: AcuityHashedSecret, timeout: T::Moment) -> DispatchResultWithPostInfo {
            let recipient = ensure_signed(origin)?;
            // Calculate lock_id.
            let lock_id = Self::get_lock_id(sender.clone(), recipient.clone(), hashed_secret, timeout);
            // Get lock value.
            let value = match <LockIdValue<T>>::get(lock_id) {
                Some(value) => value,
                None => return Err(Error::<T>::LockDoesNotExist.into()),
            };
            // Delete lock.
            LockIdValue::<T>::remove(lock_id);
            // Transfer the value back to the sender.
            T::Currency::transfer(&Self::fund_account_id(), &sender, value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Index event.
            Self::index_account(sender.clone());
            Self::index_account(recipient.clone());
            // Log info.
            Self::deposit_event(Event::DeclineByRecipient(sender, recipient, lock_id));
            Ok(().into())
        }

        #[pallet::weight(50_000_000)]
		pub fn unlock_by_sender(origin: OriginFor<T>, recipient: T::AccountId, secret: AcuitySecret, timeout: T::Moment) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Calculate hashed secret.
            let mut hashed_secret = AcuityHashedSecret::default();
            hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
            // Calculate lock_id.
            let lock_id = Self::get_lock_id(sender.clone(), recipient.clone(), hashed_secret, timeout);
            // Check lock has not timed out.
            frame_support::ensure!(timeout > <pallet_timestamp::Pallet<T>>::get(), Error::<T>::LockTimedOut);
            // Get lock value.
            let value = match <LockIdValue<T>>::get(lock_id) {
                Some(value) => value,
                None => return Err(Error::<T>::LockDoesNotExist.into()),
            };
            // Delete lock.
            LockIdValue::<T>::remove(lock_id);
            // Transfer the value.
            T::Currency::transfer(&Self::fund_account_id(), &recipient, value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Index event.
            Self::index_account(sender.clone());
            Self::index_account(recipient.clone());
            // Log info.
            Self::deposit_event(Event::UnlockBySender(sender, recipient, lock_id, secret));
            Ok(().into())
        }

        #[pallet::weight(50_000_000)]
		pub fn unlock_by_recipient(origin: OriginFor<T>, sender: T::AccountId, secret: AcuitySecret, timeout: T::Moment) -> DispatchResultWithPostInfo {
            let recipient = ensure_signed(origin)?;
            // Calculate hashed secret.
            let mut hashed_secret = AcuityHashedSecret::default();
            hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
            // Calculate lock_id.
            let lock_id = Self::get_lock_id(sender.clone(), recipient.clone(), hashed_secret, timeout);
            // Check lock has not timed out.
            frame_support::ensure!(timeout > <pallet_timestamp::Pallet<T>>::get(), Error::<T>::LockTimedOut);
            // Get lock value.
            let value = match <LockIdValue<T>>::get(lock_id) {
                Some(value) => value,
                None => return Err(Error::<T>::LockDoesNotExist.into()),
            };
            // Delete lock.
            LockIdValue::<T>::remove(lock_id);
            // Transfer the value.
            T::Currency::transfer(&Self::fund_account_id(), &recipient, value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Index event.
            Self::index_account(sender.clone());
            Self::index_account(recipient.clone());
            // Log info.
            Self::deposit_event(Event::UnlockByRecipient(sender, recipient, lock_id, secret));
            Ok(().into())
        }

        #[pallet::weight(50_000_000)]
		pub fn timeout_value(origin: OriginFor<T>, recipient: T::AccountId, hashed_secret: AcuityHashedSecret, timeout: T::Moment) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Calculate lock_id.
            let lock_id = Self::get_lock_id(sender.clone(), recipient.clone(), hashed_secret, timeout);
            // Check lock has timed out.
            frame_support::ensure!(timeout <= <pallet_timestamp::Pallet<T>>::get(), Error::<T>::LockNotTimedOut);
            // Get lock value.
            let value = match <LockIdValue<T>>::get(lock_id) {
                Some(value) => value,
                None => return Err(Error::<T>::LockDoesNotExist.into()),
            };
            // Delete lock.
            LockIdValue::<T>::remove(lock_id);
            // Transfer the value.
            T::Currency::transfer(&Self::fund_account_id(), &sender, value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Index event.
            Self::index_account(sender.clone());
            Self::index_account(recipient.clone());
            // Log info.
            Self::deposit_event(Event::Timeout(sender, recipient, lock_id));
            Ok(().into())
        }
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
        /// Value has been locked with sell asset info. \[sender, recipient, hashed_secret, timeout, value, lock_id, sell_asset_id, sell_price\]
        BuyLock(T::AccountId, T::AccountId, AcuityHashedSecret, T::Moment, BalanceOf<T>, AcuityLockId, AcuityAssetId, u128),
        /// Value has been locked. \[sender, recipient, hashed_secret, timeout, value, lock_id, buy_asset_id, buy_lock_id\]
        SellLock(T::AccountId, T::AccountId, AcuityHashedSecret, T::Moment, BalanceOf<T>, AcuityLockId, AcuityAssetId, AcuityLockId),
        /// Lock has been declined by the recipient. \[sender, recipient, lock_id\]
        DeclineByRecipient(T::AccountId, T::AccountId, AcuityLockId),
        /// Value has been unlocked by the sender. \[sender, recipient, lock_id, secret\]
        UnlockBySender(T::AccountId, T::AccountId, AcuityLockId, AcuitySecret),
        /// Value has been unlocked by the recipient. \[sender, recipient, lock_id, secret\]
        UnlockByRecipient(T::AccountId, T::AccountId, AcuityLockId, AcuitySecret),
        /// Value has been timed out. \[sender, recipient, lock_id\]
        Timeout(T::AccountId, T::AccountId, AcuityLockId),
	}

	#[pallet::error]
	pub enum Error<T> {
        /// Value must not be zero.
        ZeroValue,
        /// Value has already been locked with this lockId.
        LockAlreadyExists,
        /// No value has already been locked with this lockId.
        LockDoesNotExist,
        /// The lock has timed out.
        LockTimedOut,
        /// The lock has not timed out.
        LockNotTimedOut,
	}

    #[pallet::storage]
    #[pallet::getter(fn lock_id_value)]
    pub(super) type LockIdValue<T: Config> = StorageMap<_,
        Identity, AcuityLockId,
        BalanceOf<T>
    >;

    #[pallet::storage]
    #[pallet::getter(fn account_start_index)]
    pub(super) type AccountStartIndex<T: Config> = StorageMap<_,
        Blake2_128Concat, T::AccountId,
        u64, ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn account_next_index)]
    pub(super) type AccountNextIndex<T: Config> = StorageMap<_,
        Blake2_128Concat, T::AccountId,
        u64, ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn account_index_height)]
    pub(super) type AccountIndexHeight<T: Config> = StorageDoubleMap<_,
        Blake2_128Concat, T::AccountId,
        Blake2_128Concat, u64,
        <T as frame_system::Config>::BlockNumber
    >;

    impl<T: Config> Pallet<T> {
    	/// The account ID of the fund pot.
    	///
    	/// This actually does computation. If you need to keep using it, then make sure you cache the
    	/// value and only call this once.
    	pub fn fund_account_id() -> T::AccountId {
    		T::PalletId::get().into_account_truncating()
    	}

        pub fn get_lock_id(sender: T::AccountId, recipient: T::AccountId, hashed_secret: AcuityHashedSecret, timeout: T::Moment) -> AcuityLockId {
            let mut lock_id = AcuityLockId::default();
    		lock_id.0.copy_from_slice(&blake2_256(&[sender.encode(), recipient.encode(), hashed_secret.encode(), timeout.encode()].concat()));
            lock_id
        }

        pub fn index_account(account: T::AccountId) {
            // Get next index.
            let i = <AccountNextIndex<T>>::get(&account);
            // Insert current block number.
            <AccountIndexHeight<T>>::insert(&account, i, <frame_system::Pallet<T>>::block_number());
            // Update the next index.
            <AccountNextIndex<T>>::insert(account, i + 1);
        }

        pub fn get_index_blocks(account: T::AccountId) -> sp_std::prelude::Vec<<T as frame_system::Config>::BlockNumber> {
            let mut blocks = sp_std::prelude::Vec::new();
            let mut i = 0;
            loop {
                blocks.push(match <AccountIndexHeight<T>>::get(&account, i) {
                    Some(height) => height,
                    None => break,
                });
                i += 1;
            }
            blocks
        }
    }
}
