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

use codec::{Encode, Decode};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// An Order Id (i.e. 16 bytes).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AcuityOrderId([u8; 16]);

/// A Chain Id (i.e. 4 bytes).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AcuityChainId(u32);

/// An Adapter Id (i.e. 4 bytes).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AcuityAdapterId(u32);

/// An Asset Id (i.e. 8 bytes).
///
/// This gets serialized to the 0x-prefixed hex representation.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AcuityAssetId([u8; 8]);

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

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use super::*;

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


    #[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
    pub struct SellLock<AccountId, Balance, Moment> {
        pub order_id: AcuityOrderId,
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
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
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
	}

	#[pallet::error]
	pub enum Error<T> {
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
    #[pallet::getter(fn order_id_value)]
    pub(super) type AcuityOrderIdValues<T: Config> = StorageMap<_, Blake2_128Concat, AcuityOrderId, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn sell_lock)]
    pub(super) type SellLocks<T: Config> = StorageDoubleMap<_,
        Blake2_128Concat, AcuityOrderId,
        Blake2_128Concat, AcuityHashedSecret,
        SellLock<T::AccountId, BalanceOf<T>, T::Moment>
    >;

    #[pallet::storage]
    #[pallet::getter(fn buy_lock)]
    pub(super) type BuyLocks<T: Config> = StorageDoubleMap<_,
        Blake2_128Concat, T::AccountId,
        Blake2_128Concat, AcuityHashedSecret,
        BuyLock<T::AccountId, BalanceOf<T>, T::Moment>
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

    pub fn get_order_id(seller: T::AccountId, chain_id: AcuityChainId, adapter_id: AcuityAdapterId, asset_id: AcuityAssetId, price: u128, foreign_address: AcuityForeignAddress) -> AcuityOrderId {
        let mut order_id = AcuityOrderId::default();
		order_id.0.copy_from_slice(&blake2_128(&[seller.encode(), chain_id.encode(), adapter_id.encode(), asset_id.encode(), price.to_ne_bytes().to_vec(), foreign_address.encode()].concat()));
        order_id
    }

}
