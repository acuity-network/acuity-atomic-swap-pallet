#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_runtime::{traits::AccountIdConversion, RuntimeDebug};
use frame_support::{
    pallet_prelude::MaxEncodedLen,
    traits::{ExistenceRequirement::AllowDeath},
	traits::{Currency, Get},
    PalletId,
};
use scale_info::TypeInfo;
use sp_io::hashing::{blake2_128, keccak_256};

pub use pallet::*;

use sp_std::{
	convert::TryInto,
};


use sp_runtime::{
	traits::{
		Zero, TrailingZeroInput
	},
};


use codec::{Encode, Decode};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// An Asset Id (i.e. 8 bytes).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AcuityAssetId([u8; 16]);

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
	#[pallet::generate_store(trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
    pub trait Config: pallet_timestamp::Config + frame_system::Config {
        /// PalletId for the crowdloan pallet. An appropriate value could be ```PalletId(*b"py/cfund")```
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency type that the charity deals in
        type Currency: Currency<Self::AccountId>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

        #[pallet::weight(50_000_000)]
        pub fn deposit_stash(origin: OriginFor<T>, asset_id: AcuityAssetId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Ensure value is nonzero.
            frame_support::ensure!(!value.is_zero(), Error::<T>::ZeroValue);
            // Move the value from the sender to the pallet.
            T::Currency::transfer(&sender, &Self::fund_account_id(), value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;

            Self::stash_add(asset_id, sender, value);
            Ok(().into())
        }

        #[pallet::weight(50_000_000)]
        pub fn move_stash(origin: OriginFor<T>, from_asset_id: AcuityAssetId, to_asset_id: AcuityAssetId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Get current stash size.
            let stash_value = <StashValue<T>>::get(from_asset_id, &sender);
            // Check there is enough.
            frame_support::ensure!(stash_value >= value, Error::<T>::StashNotBigEnough);
            // Move the deposit.
            Self::stash_remove(from_asset_id, sender.clone(), value);
            Self::stash_add(to_asset_id, sender, value);
            Ok(().into())
        }

        #[pallet::weight(50_000_000)]
        pub fn withdraw_stash(origin: OriginFor<T>, asset_id: AcuityAssetId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Check there is enough.
            frame_support::ensure!(<StashValue<T>>::get(asset_id, &sender) >= value, Error::<T>::StashNotBigEnough);
            // Remove the deposit.
            Self::stash_remove(asset_id, sender.clone(), value);
            // Send the funds back.
            T::Currency::transfer(&Self::fund_account_id(), &sender, value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            Ok(().into())
        }

        #[pallet::weight(50_000_000)]
        pub fn withdraw_stash_all(origin: OriginFor<T>, asset_id: AcuityAssetId) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Get current stash size.
            let stash_value = <StashValue<T>>::get(asset_id, &sender);
            // Remove the deposit.
            Self::stash_remove(asset_id, sender.clone(), stash_value);
            // Send the funds back.
            T::Currency::transfer(&Self::fund_account_id(), &sender, stash_value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            Ok(().into())
        }

        #[pallet::weight(50_000_000)]
		pub fn lock_buy(origin: OriginFor<T>, recipient: T::AccountId, hashed_secret: AcuityHashedSecret, timeout: T::Moment, value: BalanceOf<T>, sell_asset_id: AcuityAssetId, sell_price: u128) -> DispatchResultWithPostInfo {
            let buyer = ensure_signed(origin)?;
            // Ensure value is nonzero.
            frame_support::ensure!(!value.is_zero(), Error::<T>::ZeroValue);
            // Calculate lock_id.
            let lock_id = Self::get_lock_id(buyer.clone(), recipient.clone(), hashed_secret, timeout);
            // Ensure lockId is not already in use.
            ensure!(LockIdValue::<T>::get(lock_id).is_zero(), Error::<T>::LockAlreadyExists);
            // Move the value from the sender to the pallet.
            T::Currency::transfer(&buyer, &Self::fund_account_id(), value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
                            // Move value into buy lock.
            <LockIdValue<T>>::insert(lock_id, value);
            // Log info.
            Self::deposit_event(Event::BuyLock(buyer, recipient, hashed_secret, timeout, value, lock_id, sell_asset_id, sell_price));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub fn lock_sell(origin: OriginFor<T>, recipient: T::AccountId, hashed_secret: AcuityHashedSecret, timeout: T::Moment, stash_asset_id: AcuityAssetId, value: BalanceOf<T>, buy_lock_id: AcuityLockId) -> DispatchResultWithPostInfo {
            let seller = ensure_signed(origin)?;
            // Ensure value is nonzero.
            frame_support::ensure!(!value.is_zero(), Error::<T>::ZeroValue);
            // Check there is enough.
            frame_support::ensure!(<StashValue<T>>::get(stash_asset_id, &seller) >= value, Error::<T>::StashNotBigEnough);
            // Calculate lock_id.
            let lock_id = Self::get_lock_id(seller.clone(), recipient.clone(), hashed_secret, timeout);
            // Ensure lockId is not already in use.
            ensure!(LockIdValue::<T>::get(lock_id).is_zero(), Error::<T>::LockAlreadyExists);
            // Move value into sell lock.
            Self::stash_remove(stash_asset_id, seller.clone(), value);
            <LockIdValue<T>>::insert(lock_id, value);
            // Log info.
            Self::deposit_event(Event::SellLock(seller, recipient, hashed_secret, timeout, value, lock_id, stash_asset_id, buy_lock_id));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub fn decline_by_recipient(origin: OriginFor<T>, sender: T::AccountId, hashed_secret: AcuityHashedSecret, timeout: T::Moment) -> DispatchResultWithPostInfo {
            let recipient = ensure_signed(origin)?;
            // Calculate lock_id.
            let lock_id = Self::get_lock_id(sender.clone(), recipient.clone(), hashed_secret, timeout);
            // Get lock value.
            let value = <LockIdValue<T>>::get(lock_id);
            // Delete lock.
            LockIdValue::<T>::remove(lock_id);
            // Transfer the value back to the sender.
            T::Currency::transfer(&Self::fund_account_id(), &sender, value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
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
            let value = <LockIdValue<T>>::get(lock_id);
            // Delete lock.
            LockIdValue::<T>::remove(lock_id);
            // Transfer the value.
            T::Currency::transfer(&Self::fund_account_id(), &recipient, value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
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
            let value = <LockIdValue<T>>::get(lock_id);
            // Delete lock.
            LockIdValue::<T>::remove(lock_id);
            // Transfer the value.
            T::Currency::transfer(&Self::fund_account_id(), &recipient, value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Log info.
            Self::deposit_event(Event::UnlockByRecipient(sender, recipient, lock_id, secret));
            Ok(().into())
        }

        #[pallet::weight(50_000_000)]
		pub fn timeout_stash(origin: OriginFor<T>, recipient: T::AccountId, hashed_secret: AcuityHashedSecret, timeout: T::Moment, stash_asset_id: AcuityAssetId) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Calculate lock_id.
            let lock_id = Self::get_lock_id(sender.clone(), recipient.clone(), hashed_secret, timeout);
            // Check lock has timed out.
            frame_support::ensure!(timeout <= <pallet_timestamp::Pallet<T>>::get(), Error::<T>::LockNotTimedOut);
            // Get lock value.
            let value = <LockIdValue<T>>::get(lock_id);
            // Ensure value is nonzero.
            frame_support::ensure!(!value.is_zero(), Error::<T>::ZeroValue);
            // Delete lock.
            LockIdValue::<T>::remove(lock_id);
            // Return funds.
            Self::stash_add(stash_asset_id, sender.clone(), value);
            // Log info.
            Self::deposit_event(Event::Timeout(sender, recipient, lock_id));
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
            let value = <LockIdValue<T>>::get(lock_id);
            // Delete lock.
            LockIdValue::<T>::remove(lock_id);
            // Transfer the value.
            T::Currency::transfer(&Self::fund_account_id(), &sender, value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Log info.
            Self::deposit_event(Event::Timeout(sender, recipient, lock_id));
            Ok(().into())
        }

	}
	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
        /// Value has been added to a stash. \[account, assetId, value\]
        StashAdd(T::AccountId, AcuityAssetId, BalanceOf<T>),
        /// Value has been removed from a stash. \[account, assetId, value\]
        StashRemove(T::AccountId, AcuityAssetId, BalanceOf<T>),
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
        /// The stash is not big enough.
        StashNotBigEnough,
        /// Value has already been locked with this lockId.
        LockAlreadyExists,
        /// The lock has timed out.
        LockTimedOut,
        /// The lock has not timed out.
        LockNotTimedOut,
	}

    #[pallet::storage]
    #[pallet::getter(fn stash_ll)]
    pub(super) type StashLL<T: Config> = StorageDoubleMap<_,
        Blake2_128Concat, AcuityAssetId,
        Blake2_128Concat, T::AccountId,
        T::AccountId
    >;

    #[pallet::storage]
    #[pallet::getter(fn stash_value)]
    pub(super) type StashValue<T: Config> = StorageDoubleMap<_,
        Blake2_128Concat, AcuityAssetId,
        Blake2_128Concat, T::AccountId,
        BalanceOf<T>, ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn lock_id_value)]
    pub(super) type LockIdValue<T: Config> = StorageMap<_,
        Blake2_128Concat, AcuityLockId,
        BalanceOf<T>, ValueQuery
    >;
}

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
		lock_id.0.copy_from_slice(&blake2_128(&[sender.encode(), recipient.encode(), hashed_secret.encode(), timeout.encode()].concat()));
        lock_id
    }

    pub fn stash_add(asset_id: AcuityAssetId, account: T::AccountId, value: BalanceOf<T>) {
        let zero_account_id = T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap();
        // Get new total.
        let total = <StashValue<T>>::get(asset_id, &account) + value;
        // Search for new previous.
        let mut prev = zero_account_id.clone();
        loop {
            let next = match <StashLL<T>>::get(asset_id, &prev) {
                Some(p) => p,
                None => break,
            };
            let value = <StashValue<T>>::get(asset_id, &next);
            if value < total {
                break;
            }
            prev = next;
        }
        let mut replace = false;
        // Is sender already in the list?
        if !<StashValue<T>>::get(asset_id, &account).is_zero() {
            // Search for old previous.
            let mut old_prev = zero_account_id;
            loop {
                let next = match <StashLL<T>>::get(asset_id, &old_prev) {
                    Some(p) => p,
                    None => break,
                };
                if next == account {
                    break;
                }
                old_prev = next;
            }
            // Is it in the same position?
            if prev == old_prev {
                replace = true;
            }
            else {
                // Remove sender from current position.
                let next = <StashLL<T>>::get(asset_id, &account).unwrap();
                <StashLL<T>>::insert(asset_id, old_prev, next);
            }
            if !replace {
                // Insert into linked list.
                let next = <StashLL<T>>::get(asset_id, &prev).unwrap();
                <StashLL<T>>::insert(asset_id, &account, next);
                <StashLL<T>>::insert(asset_id, &prev, &account);
            }
            // Update the value deposited.
            <StashValue<T>>::insert(asset_id, &account, total);
            // Log info.
            Self::deposit_event(Event::StashAdd(account, asset_id, value));
        }

	}

    pub fn stash_remove(asset_id: AcuityAssetId, account: T::AccountId, value: BalanceOf<T>) {
        let zero_account_id = T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap();
        // Get new total.
        let total = <StashValue<T>>::get(asset_id, &account) - value;
        // Search for old previous.
        let mut old_prev = zero_account_id.clone();
        loop {
            let next = match <StashLL<T>>::get(asset_id, &old_prev) {
                Some(p) => p,
                None => break,
            };
            if next == account {
                break;
            }
            old_prev = next;
        }
        // Is there still a stash?
        if !total.is_zero() {
            // Search for new previous.
            let mut prev = zero_account_id;
            loop {
                let next = match <StashLL<T>>::get(asset_id, &prev) {
                    Some(p) => p,
                    None => break,
                };
                let value = <StashValue<T>>::get(asset_id, &next);
                if value < total {
                    break;
                }
                prev = next;
            }
            // Is it in a new position?
            if prev != account {
                // Remove sender from old position.
                let next = <StashLL<T>>::get(asset_id, &account).unwrap();
                <StashLL<T>>::insert(asset_id, old_prev, next);
                // Insert into new position.
                let next = <StashLL<T>>::get(asset_id, &prev).unwrap();
                <StashLL<T>>::insert(asset_id, &account, next);
                <StashLL<T>>::insert(asset_id, &prev, &account);
            }
        }
        else {
            // Remove sender from current position.
            let next = <StashLL<T>>::get(asset_id, &account).unwrap();
            <StashLL<T>>::insert(asset_id, old_prev, next);
        }
        // Update the value deposited.
        <StashValue<T>>::insert(asset_id, &account, total);
        // Log info.
        Self::deposit_event(Event::StashRemove(account, asset_id, value));
	}
}
