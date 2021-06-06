#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_runtime::{
	traits::{StaticLookup, Zero, AccountIdConversion}
};
use frame_support::{
	traits::{Currency, EnsureOrigin, ReservableCurrency, OnUnbalanced, Get},
    PalletId,
};


use frame_support::{
	decl_event, decl_module, decl_storage,
	dispatch::{DispatchError, DispatchResult},
	traits::{ExistenceRequirement::AllowDeath, Imbalance},
};

use frame_support::codec::{Encode, Decode};

use sp_io::hashing::{blake2_128, blake2_256, twox_64, twox_128, twox_256};


pub use pallet::*;



#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use super::*;

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


    #[derive(Encode, Decode, Default, Clone, PartialEq)]
    pub struct SellLock {
        order_id: [u8; 16],
        value: u64,
        timeout: u32,
    }

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	pub struct Pallet<T>(PhantomData<T>);


	#[pallet::config]
	pub trait Config: frame_system::Config {
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
		pub(super) fn add_to_order(origin: OriginFor<T>, amount: BalanceOf<T>, price: u128) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let order_id: [u8; 16] = blake2_128(&[sender.encode(), price.to_ne_bytes().to_vec()].concat());

            T::Currency::transfer(&sender, &Self::fund_account_id(), amount, AllowDeath)
            				.map_err(|_| DispatchError::Other("Can't make donation"))?;

            let order_total = <OrderIdValues<T>>::get(order_id);

            <OrderIdValues<T>>::insert(order_id, order_total + price);
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub(super) fn remove_from_order(origin: OriginFor<T>, price: u64, value: u64) -> DispatchResultWithPostInfo {
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub(super) fn lock_sell(origin: OriginFor<T>, price: u64, hashed_secret: [u8; 32], timeout: u64, value: u64) -> DispatchResultWithPostInfo {
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub(super) fn unlock_sell(origin: OriginFor<T>, secret: [u8; 32]) -> DispatchResultWithPostInfo {
			Ok(().into())
		}

        #[pallet::weight(50_000_000)]
		pub(super) fn timeout_sell(origin: OriginFor<T>, price: u64, hashed_secret: [u8; 32]) -> DispatchResultWithPostInfo {
			Ok(().into())
		}
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	// pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId, Balance = BalanceOf<T> {
	pub enum Event<T: Config> {
		/// A name was set. \[who\]
		NameSet(T::AccountId),
		/// A name was forcibly set. \[target\]
		NameForced(T::AccountId),
		/// A name was changed. \[who\]
		NameChanged(T::AccountId),
	}

	/// Error for the nicks module.
	#[pallet::error]
	pub enum Error<T> {
		/// A name is too short.
		TooShort,
		/// A name is too long.
		TooLong,
		/// An account isn't named.
		Unnamed,
	}

    #[pallet::storage]
    #[pallet::getter(fn order_id_value)]
    pub(super) type OrderIdValues<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 16], u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn sell_lock)]
    pub(super) type SellLocks<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], SellLock, ValueQuery>;
}

impl<T: Config> Pallet<T> {
	/// The account ID of the fund pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn fund_account_id() -> T::AccountId {
		T::PalletId::get().into_sub_account(0)
	}
}
