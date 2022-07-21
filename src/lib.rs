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
//use sp_io::hashing::{blake2_128, keccak_256};

use std::default::Default;

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

/// A Foreign Address (i.e. 32 bytes).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AcuityForeignAddress([u8; 32]);

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



    #[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
    pub struct SellLock<AccountId, Balance, Moment> {
        pub buyer: AccountId,
        pub value: Balance,
        pub timeout: Moment,
    }

    #[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
    pub struct BuyLock<AccountId, Balance, Moment> {
        pub seller: AccountId,
        pub value: Balance,
        pub timeout: Moment,
    }

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
/*
        #[pallet::weight(50_000_000)]
        pub fn move_stash(origin: OriginFor<T>, from_asset_id: AcuityAssetId, to_asset_id: AcuityAssetId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        #[pallet::weight(50_000_000)]
        pub fn withdraw_stash(origin: OriginFor<T>, asset_id: AcuityAssetId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        #[pallet::weight(50_000_000)]
        pub fn withdraw_stash_all(origin: OriginFor<T>, asset_id: AcuityAssetId) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
*/
/*
		#[pallet::weight(50_000_000)]
		pub fn add_to_order(origin: OriginFor<T>, chain_id: AcuityChainId, adapter_id: AcuityAdapterId, asset_id: AcuityAssetId, price: u128, foreign_address: AcuityForeignAddress, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Calculate order_id.
            let order_id = Self::get_order_id(sender.clone(), chain_id, adapter_id, asset_id, price, foreign_address);
            // Move the value from the sender to the pallet.
            T::Currency::transfer(&sender, &Self::fund_account_id(), value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Add value to order.
            let order_total = <AcuityOrderIdValues<T>>::get(order_id);
            <AcuityOrderIdValues<T>>::insert(order_id, order_total + value);
            Self::deposit_event(Event::AddToOrder(sender, chain_id, adapter_id, asset_id, price, foreign_address, value));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub fn change_order(origin: OriginFor<T>, old_chain_id: AcuityChainId, old_adapter_id: AcuityAdapterId, old_asset_id: AcuityAssetId, old_price: u128, old_foreign_address: AcuityForeignAddress,
            new_chain_id: AcuityChainId, new_adapter_id: AcuityAdapterId, new_asset_id: AcuityAssetId, new_price: u128, new_foreign_address: AcuityForeignAddress, value: BalanceOf<T>) -> DispatchResultWithPostInfo
        {
            let sender = ensure_signed(origin)?;
            // Calculate order_ids.
            let old_order_id = Self::get_order_id(sender.clone(), old_chain_id, old_adapter_id, old_asset_id, old_price, old_foreign_address);
            let new_order_id = Self::get_order_id(sender.clone(), new_chain_id, new_adapter_id, new_asset_id, new_price, new_foreign_address);
            // Transfer value.
            let order_value = <AcuityOrderIdValues<T>>::get(old_order_id);
            frame_support::ensure!(value <= order_value, Error::<T>::OrderTooSmall);
            <AcuityOrderIdValues<T>>::insert(old_order_id, order_value - value);
            let order_value = <AcuityOrderIdValues<T>>::get(new_order_id);
            <AcuityOrderIdValues<T>>::insert(new_order_id, order_value + value);
            // Log info.
            Self::deposit_event(Event::RemoveFromOrder(sender.clone(), old_chain_id, old_adapter_id, old_asset_id, old_price, old_foreign_address, value));
            Self::deposit_event(Event::AddToOrder(sender, new_chain_id, new_adapter_id, new_asset_id, new_price, new_foreign_address, value));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
        pub fn change_order_all(origin: OriginFor<T>, old_chain_id: AcuityChainId, old_adapter_id: AcuityAdapterId, old_asset_id: AcuityAssetId, old_price: u128, old_foreign_address: AcuityForeignAddress,
            new_chain_id: AcuityChainId, new_adapter_id: AcuityAdapterId, new_asset_id: AcuityAssetId, new_price: u128, new_foreign_address: AcuityForeignAddress) -> DispatchResultWithPostInfo
        {
            let sender = ensure_signed(origin)?;
            // Calculate order_ids.
            let old_order_id = Self::get_order_id(sender.clone(), old_chain_id, old_adapter_id, old_asset_id, old_price, old_foreign_address);
            let new_order_id = Self::get_order_id(sender.clone(), new_chain_id, new_adapter_id, new_asset_id, new_price, new_foreign_address);
            // Transfer value.
            let old_order_value = <AcuityOrderIdValues<T>>::get(old_order_id);
            <AcuityOrderIdValues<T>>::remove(old_order_id);
            let new_order_value = <AcuityOrderIdValues<T>>::get(new_order_id);
            <AcuityOrderIdValues<T>>::insert(new_order_id, old_order_value + new_order_value);
            // Log info.
            Self::deposit_event(Event::RemoveFromOrder(sender.clone(), old_chain_id, old_adapter_id, old_asset_id, old_price, old_foreign_address, old_order_value));
            Self::deposit_event(Event::AddToOrder(sender, new_chain_id, new_adapter_id, new_asset_id, new_price, new_foreign_address, old_order_value));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub fn remove_from_order(origin: OriginFor<T>, chain_id: AcuityChainId, adapter_id: AcuityAdapterId, asset_id: AcuityAssetId, price: u128, foreign_address: AcuityForeignAddress, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Calculate order_id.
            let order_id = Self::get_order_id(sender.clone(), chain_id, adapter_id, asset_id, price, foreign_address);
            // Check there is enough.
            let order_value = <AcuityOrderIdValues<T>>::get(order_id);
            frame_support::ensure!(value <= order_value, Error::<T>::OrderTooSmall);
            // Move the value from the pallet to the sender.
            T::Currency::transfer(&Self::fund_account_id(), &sender, value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Remove value from order.
            <AcuityOrderIdValues<T>>::insert(order_id, order_value - value);
            Self::deposit_event(Event::RemoveFromOrder(sender, chain_id, adapter_id, asset_id, price, foreign_address, value));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
        pub fn remove_from_order_all(origin: OriginFor<T>, chain_id: AcuityChainId, adapter_id: AcuityAdapterId, asset_id: AcuityAssetId, price: u128, foreign_address: AcuityForeignAddress) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Calculate order_id.
            let order_id = Self::get_order_id(sender.clone(), chain_id, adapter_id, asset_id, price, foreign_address);
            let value = <AcuityOrderIdValues<T>>::get(order_id);
            // Move the value from the pallet to the sender.
            T::Currency::transfer(&Self::fund_account_id(), &sender, value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Remove value from order.
            <AcuityOrderIdValues<T>>::remove(order_id);
            Self::deposit_event(Event::RemoveFromOrder(sender, chain_id, adapter_id, asset_id, price, foreign_address, value));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub fn lock_sell(origin: OriginFor<T>, hashed_secret: AcuityHashedSecret, chain_id: AcuityChainId, adapter_id: AcuityAdapterId, asset_id: AcuityAssetId, price: u128, foreign_address: AcuityForeignAddress, buyer: T::AccountId, value: BalanceOf<T>, timeout: T::Moment) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Calculate order_id.
            let order_id = Self::get_order_id(sender.clone(), chain_id, adapter_id, asset_id, price, foreign_address);
            // Check there is enough.
            let order_total = <AcuityOrderIdValues<T>>::get(order_id);
            frame_support::ensure!(value <= order_total, Error::<T>::OrderTooSmall);
            // Ensure hashed secret is not already in use.
            ensure!(
				!SellLocks::<T>::contains_key(order_id, hashed_secret),
				Error::<T>::HashedSecretAlreadyInUse
			);
            // Move value into sell lock.
            <AcuityOrderIdValues<T>>::insert(order_id, order_total - value);

            let sell_lock: SellLock<T::AccountId, BalanceOf<T>, T::Moment> = SellLock {
                order_id: order_id,
                buyer: buyer,
                value: value,
                timeout: timeout,
            };
            SellLocks::<T>::insert(order_id, hashed_secret, sell_lock);
            Self::deposit_event(Event::LockSell(order_id, hashed_secret, timeout, value));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub fn unlock_sell(origin: OriginFor<T>, order_id: AcuityOrderId, secret: AcuitySecret) -> DispatchResultWithPostInfo {
            let _sender = ensure_signed(origin)?;
			let now = <pallet_timestamp::Pallet<T>>::get();
            // Calculate hashed secret.
            let mut hashed_secret = AcuityHashedSecret::default();
            hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
            // Check sell lock has not timed out.
            let lock = SellLocks::<T>::get(order_id, hashed_secret).ok_or(Error::<T>::LockNotFound)?;
            frame_support::ensure!(lock.timeout > now, Error::<T>::LockTimedOut);
            // Delete lock.
            SellLocks::<T>::remove(order_id, hashed_secret);
            // Send the funds.
            T::Currency::transfer(&Self::fund_account_id(), &lock.buyer, lock.value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            Self::deposit_event(Event::UnlockSell(order_id, secret, lock.buyer));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub fn timeout_sell(origin: OriginFor<T>, hashed_secret: AcuityHashedSecret, chain_id: AcuityChainId, adapter_id: AcuityAdapterId, asset_id: AcuityAssetId, price: u128, foreign_address: AcuityForeignAddress) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let now = <pallet_timestamp::Pallet<T>>::get();
            // Calculate order_id.
            let order_id = Self::get_order_id(sender.clone(), chain_id, adapter_id, asset_id, price, foreign_address);
            // Check order_id is correct and lock has timed out.
            let lock = SellLocks::<T>::get(order_id, hashed_secret).ok_or(Error::<T>::LockNotFound)?;
            frame_support::ensure!(lock.order_id == order_id, Error::<T>::WrongOrderId);
            frame_support::ensure!(lock.timeout <= now, Error::<T>::LockNotTimedOut);
            // Delete lock.
            SellLocks::<T>::remove(order_id, hashed_secret);
            // Return funds to sell order.
            let order_total = <AcuityOrderIdValues<T>>::get(order_id);
            <AcuityOrderIdValues<T>>::insert(order_id, order_total + lock.value);
            Self::deposit_event(Event::TimeoutSell(order_id, hashed_secret));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub fn lock_buy(origin: OriginFor<T>, hashed_secret: AcuityHashedSecret, chain_id: AcuityChainId, adapter_id: AcuityAdapterId, order_id: AcuityOrderId, seller: T::AccountId, timeout: T::Moment, value: BalanceOf<T>, foreign_address: AcuityForeignAddress) -> DispatchResultWithPostInfo {
            let buyer = ensure_signed(origin)?;
            // Ensure hashed secret is not already in use.
            ensure!(
				!BuyLocks::<T>::contains_key(&buyer, hashed_secret),
				Error::<T>::HashedSecretAlreadyInUse
			);
            // Move the value from the sender to the pallet.
            T::Currency::transfer(&buyer, &Self::fund_account_id(), value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Store lock data.
            let lock: BuyLock<T::AccountId, BalanceOf<T>, T::Moment> = BuyLock {
                seller: seller.clone(),
                value: value,
                timeout: timeout,
            };
            <BuyLocks<T>>::insert(&buyer, hashed_secret, lock);

            Self::deposit_event(Event::LockBuy(buyer, seller, hashed_secret, timeout, value, chain_id, adapter_id, order_id, foreign_address));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub fn unlock_buy(origin: OriginFor<T>, buyer: T::AccountId, secret: AcuitySecret) -> DispatchResultWithPostInfo {
            let _sender = ensure_signed(origin)?;
            let now = <pallet_timestamp::Pallet<T>>::get();
            // Calculate hashed secret.
            let mut hashed_secret = AcuityHashedSecret::default();
            hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
            // Check lock has not timed out.
            let lock = <BuyLocks<T>>::get(&buyer, hashed_secret).ok_or(Error::<T>::LockNotFound)?;
            frame_support::ensure!(lock.timeout > now, Error::<T>::LockTimedOut);
            // Delete lock.
            <BuyLocks<T>>::remove(&buyer, hashed_secret);
            // Send the funds.
            T::Currency::transfer(&Self::fund_account_id(), &lock.seller, lock.value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            Self::deposit_event(Event::UnlockBuy(buyer, hashed_secret));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub fn timeout_buy(origin: OriginFor<T>, secret: AcuitySecret) -> DispatchResultWithPostInfo {
            let buyer = ensure_signed(origin)?;
            let now = <pallet_timestamp::Pallet<T>>::get();
            // Calculate hashed secret.
            let mut hashed_secret = AcuityHashedSecret::default();
            hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
            // Check lock has timed out.
            let lock = <BuyLocks<T>>::get(&buyer, hashed_secret).ok_or(Error::<T>::LockNotFound)?;
            frame_support::ensure!(lock.timeout <= now, Error::<T>::LockNotTimedOut);
            // Delete lock.
            <BuyLocks<T>>::remove(&buyer, hashed_secret);
            // Send the funds.
            T::Currency::transfer(&Self::fund_account_id(), &buyer, lock.value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            Self::deposit_event(Event::TimeoutBuy(buyer, hashed_secret));
			Ok(().into())
		}
        */
	}
	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
        /*
        /// Value was added to a sell order. \[seller, chain_id, adapter_id, asset_id, price, foreign_address, value\]
        AddToOrder(T::AccountId, AcuityChainId, AcuityAdapterId, AcuityAssetId, u128, AcuityForeignAddress, BalanceOf<T>),
        /// Value was removed from a sell order. \[seller, chain_id, adapter_id, asset_id, price, foreign_address, value\]
        RemoveFromOrder(T::AccountId, AcuityChainId, AcuityAdapterId, AcuityAssetId, u128, AcuityForeignAddress, BalanceOf<T>),
        /// A sell lock was created. \[order_id, hashed_secret, timeout, value\]
        LockSell(AcuityOrderId, AcuityHashedSecret, T::Moment, BalanceOf<T>),
        /// A sell lock was unlocked \[order_id, secret, buyer\]
        UnlockSell(AcuityOrderId, AcuitySecret, T::AccountId),
        /// A sell lock was timed out. \[order_id, hashed_secret\]
        TimeoutSell(AcuityOrderId, AcuityHashedSecret),
        /// A buy lock was created. \[buyer, seller, hashed_secret, timeout, value, chain_id, adapter_id, order_id, foreign_address\]
        LockBuy(T::AccountId, T::AccountId, AcuityHashedSecret, T::Moment, BalanceOf<T>, AcuityChainId, AcuityAdapterId, AcuityOrderId, AcuityForeignAddress),
        /// A buy lock was unlocked. \[buyer, hashed_secret\]
        UnlockBuy(T::AccountId, AcuityHashedSecret),
        /// A buy lock was timed out. \[buyer, hashed_secret\]
        TimeoutBuy(T::AccountId, AcuityHashedSecret),
        */
	}

	#[pallet::error]
	pub enum Error<T> {
        /// Value must not be zero.
        ZeroValue,
        /// The order has too little value.
        OrderTooSmall,
        /// The order ID is incorrect.
        WrongOrderId,
        /// The lock has timed out.
        LockNotFound,
        /// The lock has timed out.
        LockTimedOut,
        /// The lock has not timed out.
        LockNotTimedOut,
        /// The hashed secret is already in use.
        HashedSecretAlreadyInUse,
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
    pub(super) type AcuityLockIdValue<T: Config> = StorageMap<_,
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
                let next =  <StashLL<T>>::get(asset_id, &account).unwrap();
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
//            emit StashAdd(msg.sender, assetId, value);
        }

	}

    pub fn stash_remove() {
	}
}
