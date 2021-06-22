#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_runtime::traits::AccountIdConversion;
use frame_support::{
    traits::{ExistenceRequirement::AllowDeath},
	traits::{Currency, Get},
    PalletId,
};

use sp_io::hashing::{blake2_128, keccak_256};

pub use pallet::*;

use sp_std::{
	convert::TryInto,
};

use codec::Encode;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use super::*;

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


    #[derive(Encode, Decode, Default, Clone, PartialEq)]
    pub struct SellLock<Balance, Moment> {
        pub order_id: [u8; 16],
        pub value: Balance,
        pub timeout: Moment,
    }

    #[derive(Encode, Decode, Default, Clone, PartialEq)]
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
		pub(super) fn add_to_order(origin: OriginFor<T>, asset_id: [u8; 16], price: u128, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Calculate order_id.
            let order_id: [u8; 16] = Self::get_order_id(sender.clone(), asset_id, price);
            // Move the value from the sender to the pallet.
            T::Currency::transfer(&sender, &Self::fund_account_id(), value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Add value to order.
            let order_total = <OrderIdValues<T>>::get(order_id);
            <OrderIdValues<T>>::insert(order_id, order_total + value);
            Self::deposit_event(Event::AddToOrder(sender, asset_id, price, value));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub(super) fn change_order(origin: OriginFor<T>, old_asset_id: [u8; 16], old_price: u128, new_asset_id: [u8; 16], new_price: u128, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Calculate order_ids.
            let old_order_id: [u8; 16] = Self::get_order_id(sender.clone(), old_asset_id, old_price);
            let new_order_id: [u8; 16] = Self::get_order_id(sender.clone(), new_asset_id, new_price);
            // Transfer value.
            let order_value = <OrderIdValues<T>>::get(old_order_id);
            frame_support::ensure!(value <= order_value, Error::<T>::OrderTooSmall);
            <OrderIdValues<T>>::insert(old_order_id, order_value - value);
            let order_value = <OrderIdValues<T>>::get(new_order_id);
            <OrderIdValues<T>>::insert(new_order_id, order_value + value);
            // Log info.
            Self::deposit_event(Event::RemoveFromOrder(sender.clone(), old_asset_id, old_price, value));
            Self::deposit_event(Event::AddToOrder(sender, new_asset_id, new_price, value));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub(super) fn change_order_all(origin: OriginFor<T>, old_asset_id: [u8; 16], old_price: u128, new_asset_id: [u8; 16], new_price: u128) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Calculate order_ids.
            let old_order_id: [u8; 16] = Self::get_order_id(sender.clone(), old_asset_id, old_price);
            let new_order_id: [u8; 16] = Self::get_order_id(sender.clone(), new_asset_id, new_price);
            // Transfer value.
            let old_order_value = <OrderIdValues<T>>::get(old_order_id);
            <OrderIdValues<T>>::remove(old_order_id);
            let new_order_value = <OrderIdValues<T>>::get(new_order_id);
            <OrderIdValues<T>>::insert(new_order_id, old_order_value + new_order_value);
            // Log info.
            Self::deposit_event(Event::RemoveFromOrder(sender.clone(), old_asset_id, old_price, old_order_value));
            Self::deposit_event(Event::AddToOrder(sender, new_asset_id, new_price, new_order_value));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub(super) fn remove_from_order(origin: OriginFor<T>, asset_id: [u8; 16], price: u128, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Calculate order_id.
            let order_id: [u8; 16] = Self::get_order_id(sender.clone(), asset_id, price);
            // Check there is enough.
            let order_value = <OrderIdValues<T>>::get(order_id);
            frame_support::ensure!(value <= order_value, Error::<T>::OrderTooSmall);
            // Move the value from the pallet to the sender.
            T::Currency::transfer(&Self::fund_account_id(), &sender, value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Remove value from order.
            <OrderIdValues<T>>::insert(order_id, order_value - value);
            Self::deposit_event(Event::RemoveFromOrder(sender, asset_id, price, value));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub(super) fn remove_from_order_all(origin: OriginFor<T>, asset_id: [u8; 16], price: u128) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Calculate order_id.
            let order_id: [u8; 16] = Self::get_order_id(sender.clone(), asset_id, price);
            let value = <OrderIdValues<T>>::get(order_id);
            // Move the value from the pallet to the sender.
            T::Currency::transfer(&Self::fund_account_id(), &sender, value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Remove value from order.
            <OrderIdValues<T>>::remove(order_id);
            Self::deposit_event(Event::RemoveFromOrder(sender, asset_id, price, value));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub(super) fn lock_sell(origin: OriginFor<T>, hashed_secret: [u8; 32], asset_id: [u8; 16], price: u128, value: BalanceOf<T>, timeout: T::Moment) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Calculate order_id.
            let order_id: [u8; 16] = Self::get_order_id(sender.clone(), asset_id, price);
            // Check there is enough.
            let order_total = <OrderIdValues<T>>::get(order_id);
            frame_support::ensure!(value <= order_total, Error::<T>::OrderTooSmall);
            // Ensure hashed secret is not already in use.
            let lock = <SellLocks<T>>::get(hashed_secret);
            frame_support::ensure!(TryInto::<u64>::try_into(lock.value).ok() == Some(0), Error::<T>::HashedSecretAlreadyInUse);
            // Move value into sell lock.
            <OrderIdValues<T>>::insert(order_id, order_total - value);

            let sell_lock: SellLock<BalanceOf<T>, T::Moment> = SellLock {
                order_id: order_id,
                value: value,
                timeout: timeout,
            };
            <SellLocks<T>>::insert(hashed_secret, sell_lock);
            Self::deposit_event(Event::LockSell(hashed_secret, order_id, value, timeout));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub(super) fn unlock_sell(origin: OriginFor<T>, secret: [u8; 32]) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
			let _now = <pallet_timestamp::Pallet<T>>::get();
            // Calculate hashed secret.
            let hashed_secret: [u8; 32] = keccak_256(&secret);
            // Check sell lock has not timed out.
            let lock = <SellLocks<T>>::get(hashed_secret);
            frame_support::ensure!(lock.timeout > _now, Error::<T>::LockTimedOut);
            // Delete lock.
            <SellLocks<T>>::remove(hashed_secret);
            // Send the funds.
            T::Currency::transfer(&Self::fund_account_id(), &sender, lock.value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            Self::deposit_event(Event::UnlockSell(secret, sender));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub(super) fn timeout_sell(origin: OriginFor<T>, hashed_secret: [u8; 32], asset_id: [u8; 16], price: u128) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let _now = <pallet_timestamp::Pallet<T>>::get();
            // Calculate order_id.
            let order_id: [u8; 16] = Self::get_order_id(sender.clone(), asset_id, price);
            // Check order_id is correct and lock has timed out.
            let lock = <SellLocks<T>>::get(hashed_secret);
            frame_support::ensure!(lock.order_id == order_id, Error::<T>::WrongOrderId);
            frame_support::ensure!(lock.timeout <= _now, Error::<T>::LockNotTimedOut);
            // Delete lock.
            <SellLocks<T>>::remove(hashed_secret);
            // Return funds to sell order.
            let order_total = <OrderIdValues<T>>::get(order_id);
            <OrderIdValues<T>>::insert(order_id, order_total + lock.value);
            Self::deposit_event(Event::TimeoutSell(hashed_secret));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub(super) fn lock_buy(origin: OriginFor<T>, hashed_secret: [u8; 32], asset_id: [u8; 16], order_id: [u8; 16], seller: T::AccountId, timeout: T::Moment, value: BalanceOf<T>, ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            // Ensure hashed secret is not already in use.
            let lock = <BuyLocks<T>>::get(hashed_secret);
            frame_support::ensure!(TryInto::<u64>::try_into(lock.value).ok() == Some(0), Error::<T>::HashedSecretAlreadyInUse);
            // Move the value from the sender to the pallet.
            T::Currency::transfer(&sender, &Self::fund_account_id(), value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            // Store lock data.
            let lock: BuyLock<T::AccountId, BalanceOf<T>, T::Moment> = BuyLock {
                seller: seller.clone(),
                value: value,
                timeout: timeout,
            };
            <BuyLocks<T>>::insert(hashed_secret, lock);
            Self::deposit_event(Event::LockBuy(hashed_secret, asset_id, order_id, seller, value, timeout));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub(super) fn unlock_buy(origin: OriginFor<T>, secret: [u8; 32]) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let _now = <pallet_timestamp::Pallet<T>>::get();
            // Calculate hashed secret.
            let hashed_secret: [u8; 32] = keccak_256(&secret);
            // Check lock has not timed out.
            let lock = <BuyLocks<T>>::get(hashed_secret);
            frame_support::ensure!(lock.timeout > _now, Error::<T>::LockTimedOut);
            // Delete lock.
            <BuyLocks<T>>::remove(hashed_secret);
            // Send the funds.
            T::Currency::transfer(&Self::fund_account_id(), &lock.seller, lock.value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            Self::deposit_event(Event::UnlockBuy(hashed_secret));
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub(super) fn timeout_buy(origin: OriginFor<T>, secret: [u8; 32]) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let _now = <pallet_timestamp::Pallet<T>>::get();
            // Calculate hashed secret.
            let hashed_secret: [u8; 32] = keccak_256(&secret);
            // Check lock has timed out.
            let lock = <BuyLocks<T>>::get(hashed_secret);
            frame_support::ensure!(lock.timeout <= _now, Error::<T>::LockNotTimedOut);
            // Delete lock.
            <BuyLocks<T>>::remove(hashed_secret);
            // Send the funds.
            T::Currency::transfer(&Self::fund_account_id(), &sender, lock.value, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't transfer value."))?;
            Self::deposit_event(Event::TimeoutBuy(hashed_secret));
			Ok(().into())
		}
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Value", T::Moment = "Timestamp")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        // Value was added to a sell order. \[seller\], \[asset_id\], \[price\], \[value\]
        AddToOrder(T::AccountId, [u8; 16], u128, BalanceOf<T>),
        // Value was removed from a sell order. \[seller\], \[asset_id\], \[price\], \[value\]
        RemoveFromOrder(T::AccountId, [u8; 16], u128, BalanceOf<T>),
        // A sell lock was created. \[hashed_secret\], \[order_id\], \[value\], \[timeout\]
        LockSell([u8; 32], [u8; 16], BalanceOf<T>, T::Moment),
        // A sell lock was unlocked \[secret\], \[buyer\]
        UnlockSell([u8; 32], T::AccountId),
        // A sell lock was timed out. \[hashed_secret\]
        TimeoutSell([u8; 32]),
        // A buy lock was created. \[order_id\], \[hashed_secret\], \[seller\], \[value\], \[timeout\]
        LockBuy([u8; 32], [u8; 16], [u8; 16], T::AccountId, BalanceOf<T>, T::Moment),
        // A buy lock was unlocked. \[hashed_secret\]
        UnlockBuy([u8; 32]),
        // A buy lock was timed out. \[hashed_secret\]
        TimeoutBuy([u8; 32]),
	}

	/// Error for the nicks module.
	#[pallet::error]
	pub enum Error<T> {
        // The order has too little value.
        OrderTooSmall,
        // The order ID is incorrect.
        WrongOrderId,
        // The lock has timed out.
        LockTimedOut,
        // The lock has not timed out.
        LockNotTimedOut,
        // The hashed secret is already in use.
        HashedSecretAlreadyInUse,
	}

    #[pallet::storage]
    #[pallet::getter(fn order_id_value)]
    pub(super) type OrderIdValues<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 16], BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn sell_lock)]
    pub(super) type SellLocks<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], SellLock<BalanceOf<T>, T::Moment>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn buy_lock)]
    pub(super) type BuyLocks<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], BuyLock<T::AccountId, BalanceOf<T>, T::Moment>, ValueQuery>;
}

impl<T: Config> Pallet<T> {
	/// The account ID of the fund pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn fund_account_id() -> T::AccountId {
		T::PalletId::get().into_sub_account(0)
	}

    pub fn get_order_id(seller: T::AccountId, asset_id: [u8; 16], price: u128 ) -> [u8; 16] {
        blake2_128(&[seller.encode(), asset_id.to_vec(), price.to_ne_bytes().to_vec()].concat())
    }

}
