use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use sp_io::hashing::{keccak_256};
use super::*;


const A: u64 = 1;
const B: u64 = 2;
const C: u64 = 3;
const D: u64 = 4;

#[test]
fn deposit_stash_control_zero_value() {
	new_test_ext().execute_with(|| {
		let mut asset_id = AcuityAssetId::default();
		asset_id.0.copy_from_slice(&[1; 16]);
		assert_ok!(AcuityAtomicSwap::deposit_stash(Origin::signed(A), asset_id, 50));
	});
}

#[test]
fn deposit_stash_fail_zero_value() {
	new_test_ext().execute_with(|| {
		let mut asset_id = AcuityAssetId::default();
		asset_id.0.copy_from_slice(&[1; 16]);
		assert_noop!(
			AcuityAtomicSwap::deposit_stash(Origin::signed(A), asset_id, 0),
			Error::<Test>::ZeroValue,
		);
	});
}

#[test]
fn deposit_stash() {
	new_test_ext().execute_with(|| {
		let mut asset_id = AcuityAssetId::default();
		asset_id.0.copy_from_slice(&[1; 16]);
		let stashes = AcuityAtomicSwap::get_stashes(asset_id, 0, 100);
		assert_eq!(stashes.len(), 0);

		assert_ok!(AcuityAtomicSwap::deposit_stash(Origin::signed(A), asset_id, 50));
		let stashes = AcuityAtomicSwap::get_stashes(asset_id, 0, 100);
		assert_eq!(stashes.len(), 1);
		assert_eq!(stashes[0], (A, 50));

		assert_ok!(AcuityAtomicSwap::deposit_stash(Origin::signed(B), asset_id, 40));
		let stashes = AcuityAtomicSwap::get_stashes(asset_id, 0, 100);
		assert_eq!(stashes.len(), 2);
		assert_eq!(stashes[0], (A, 50));
		assert_eq!(stashes[1], (B, 40));

		assert_ok!(AcuityAtomicSwap::deposit_stash(Origin::signed(C), asset_id, 60));
		let stashes = AcuityAtomicSwap::get_stashes(asset_id, 0, 100);
		assert_eq!(stashes.len(), 3);
		assert_eq!(stashes[0], (C, 60));
		assert_eq!(stashes[1], (A, 50));
		assert_eq!(stashes[2], (B, 40));

		assert_ok!(AcuityAtomicSwap::deposit_stash(Origin::signed(D), asset_id, 45));
		let stashes = AcuityAtomicSwap::get_stashes(asset_id, 0, 100);
		assert_eq!(stashes.len(), 4);
		assert_eq!(stashes[0], (C, 60));
		assert_eq!(stashes[1], (A, 50));
		assert_eq!(stashes[2], (D, 45));
		assert_eq!(stashes[3], (B, 40));

		assert_ok!(AcuityAtomicSwap::deposit_stash(Origin::signed(A), asset_id, 10));
		let stashes = AcuityAtomicSwap::get_stashes(asset_id, 0, 100);
		assert_eq!(stashes.len(), 4);
		assert_eq!(stashes[0], (C, 60));
		assert_eq!(stashes[1], (A, 60));
		assert_eq!(stashes[2], (D, 45));
		assert_eq!(stashes[3], (B, 40));

		assert_ok!(AcuityAtomicSwap::deposit_stash(Origin::signed(A), asset_id, 1));
		let stashes = AcuityAtomicSwap::get_stashes(asset_id, 0, 100);
		assert_eq!(stashes.len(), 4);
		assert_eq!(stashes[0], (A, 61));
		assert_eq!(stashes[1], (C, 60));
		assert_eq!(stashes[2], (D, 45));
		assert_eq!(stashes[3], (B, 40));
	});
}

/*
#[test]
fn lock_sell_control_too_small() {
    new_test_ext().execute_with(|| {
        let price: u128 = 5;
        assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), 50));
        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), AcuityHashedSecret::default(), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 50, 0));
	});
}

#[test]
fn lock_sell_fail_too_small() {
    new_test_ext().execute_with(|| {
        let price: u128 = 5;
        assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), 50));
		assert_noop!(
            AcuityAtomicSwap::lock_sell(Origin::signed(A), AcuityHashedSecret::default(), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 60, 0),
			Error::<Test>::OrderTooSmall
		);
	});
}

#[test]
fn lock_sell_control_already_in_use() {
    new_test_ext().execute_with(|| {
        let price: u128 = 5;
        let mut hashed_secret_new = AcuityHashedSecret::default();
        hashed_secret_new.0.copy_from_slice(&[1; 32]);
        assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), 50));
        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), AcuityHashedSecret::default(), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 10, 0));
        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), hashed_secret_new, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 10, 0));
	});
}

#[test]
fn lock_sell_fail_already_in_use() {
    new_test_ext().execute_with(|| {
        let price: u128 = 5;
        assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), 50));
        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), AcuityHashedSecret::default(), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 10, 0));
		assert_noop!(
            AcuityAtomicSwap::lock_sell(Origin::signed(A), AcuityHashedSecret::default(), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 10, 0),
			Error::<Test>::HashedSecretAlreadyInUse
		);
	});
}

#[test]
fn lock_sell() {
	new_test_ext().execute_with(|| {
        let price: u128 = 5;
		assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), 50));

        assert_eq!(Balances::free_balance(A), 50);
        assert_eq!(Balances::free_balance(AcuityAtomicSwap::fund_account_id()), 50);

        let order_id = AcuityAtomicSwap::get_order_id(A, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default());

        assert_eq!(AcuityAtomicSwap::order_id_value(order_id), 50);

        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), AcuityHashedSecret::default(), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 10, 0));
        assert_eq!(AcuityAtomicSwap::order_id_value(order_id), 40);

        let lock = AcuityAtomicSwap::sell_lock(order_id, AcuityHashedSecret::default()).unwrap();
        assert_eq!(lock.order_id, order_id);
        assert_eq!(lock.value, 10);
        assert_eq!(lock.timeout, 0);
	});
}

#[test]
fn unlock_sell_control_timed_out() {
    new_test_ext().execute_with(|| {
        let price: u128 = 5;
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let order_id = AcuityAtomicSwap::get_order_id(A, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default());
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), 50));
        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 10, now + 1000));
        assert_ok!(AcuityAtomicSwap::unlock_sell(Origin::signed(B), order_id, secret));
	});
}

#[test]
fn unlock_sell_fail_timed_out() {
    new_test_ext().execute_with(|| {
        let price: u128 = 5;
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let order_id = AcuityAtomicSwap::get_order_id(A, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default());
        let now = <pallet_timestamp::Pallet<Test>>::get();
		assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), 50));
        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 10, now));
		assert_noop!(
            AcuityAtomicSwap::unlock_sell(Origin::signed(B), order_id, secret),
			Error::<Test>::LockTimedOut
		);
	});
}

#[test]
fn unlock_sell() {
	new_test_ext().execute_with(|| {
        let price: u128 = 5;
        let mut secret = AcuitySecret::default();
        secret.0.copy_from_slice(&hex::decode("4b1694df15172648181bcb37868b25d3bd9ff95d0f10ec150f783802a81a07fb").unwrap());
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&hex::decode("094cd46013683e3929f474bf04e9ff626a6d7332c195dfe014e4b4a3fbb3ea54").unwrap());
        let now = <pallet_timestamp::Pallet<Test>>::get();
		assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), 50));

        assert_eq!(Balances::free_balance(A), 50);
        assert_eq!(Balances::free_balance(B), 200);
        assert_eq!(Balances::free_balance(AcuityAtomicSwap::fund_account_id()), 50);

        let order_id = AcuityAtomicSwap::get_order_id(A, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default());

        assert_eq!(AcuityAtomicSwap::order_id_value(order_id), 50);

        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 10, now + 1000));
        assert_eq!(AcuityAtomicSwap::order_id_value(order_id), 40);

        let lock = AcuityAtomicSwap::sell_lock(order_id, hashed_secret).unwrap();
        assert_eq!(lock.order_id, order_id);
        assert_eq!(lock.value, 10);
        assert_eq!(lock.timeout, now + 1000);

        assert_ok!(AcuityAtomicSwap::unlock_sell(Origin::signed(B), order_id, secret));
        assert_eq!(AcuityAtomicSwap::order_id_value(order_id), 40);

        assert_eq!(AcuityAtomicSwap::sell_lock(order_id, hashed_secret), None);

        assert_eq!(Balances::free_balance(A), 50);
        assert_eq!(Balances::free_balance(B), 210);
        assert_eq!(Balances::free_balance(AcuityAtomicSwap::fund_account_id()), 40);

	});
}

#[test]
fn timeout_sell_control_wrong_order_id() {
    new_test_ext().execute_with(|| {
        let price: u128 = 5;
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), 50));
        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 10, now));
        assert_ok!(AcuityAtomicSwap::timeout_sell(Origin::signed(A), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default()));
	});
}

#[test]
fn timeout_sell_fail_wrong_order_id() {
    new_test_ext().execute_with(|| {
        let price: u128 = 5;
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), 50));
        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), AcuityHashedSecret::default(), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 10, now));
		assert_noop!(
            AcuityAtomicSwap::timeout_sell(Origin::signed(A), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price - 1, AcuityForeignAddress::default()),
			Error::<Test>::LockNotFound
		);
	});
}

#[test]
fn timeout_sell_control_not_timed_out() {
    new_test_ext().execute_with(|| {
        let price: u128 = 5;
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
		assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), 50));
        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 10, now));
        assert_ok!(AcuityAtomicSwap::timeout_sell(Origin::signed(A), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default()));
	});
}

#[test]
fn timeout_sell_fail_not_timed_out() {
    new_test_ext().execute_with(|| {
        let price: u128 = 5;
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
		assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), 50));
        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 10, now + 1000));
		assert_noop!(
            AcuityAtomicSwap::timeout_sell(Origin::signed(A), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default()),
			Error::<Test>::LockNotTimedOut
		);
	});
}

#[test]
fn timeout_sell() {
	new_test_ext().execute_with(|| {
        let price: u128 = 5;
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
		assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), 50));

        assert_eq!(Balances::free_balance(A), 50);
        assert_eq!(Balances::free_balance(B), 200);
        assert_eq!(Balances::free_balance(AcuityAtomicSwap::fund_account_id()), 50);

        let order_id = AcuityAtomicSwap::get_order_id(A, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default());

        assert_eq!(AcuityAtomicSwap::order_id_value(order_id), 50);

        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default(), B, 10, now));
        assert_eq!(AcuityAtomicSwap::order_id_value(order_id), 40);

        let lock = AcuityAtomicSwap::sell_lock(order_id, hashed_secret).unwrap();
        assert_eq!(lock.order_id, order_id);
        assert_eq!(lock.value, 10);
        assert_eq!(lock.timeout, now);

        assert_ok!(AcuityAtomicSwap::timeout_sell(Origin::signed(A), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityAssetId::default(), price, AcuityForeignAddress::default()));
        assert_eq!(AcuityAtomicSwap::order_id_value(order_id), 50);

        assert_eq!(AcuityAtomicSwap::sell_lock(order_id, hashed_secret), None);

        assert_eq!(Balances::free_balance(A), 50);
        assert_eq!(Balances::free_balance(B), 200);
        assert_eq!(Balances::free_balance(AcuityAtomicSwap::fund_account_id()), 50);
	});
}

#[test]
fn lock_buy_control_already_in_use() {
    new_test_ext().execute_with(|| {
        let now = <pallet_timestamp::Pallet<Test>>::get();
        let mut hashed_secret_new = AcuityHashedSecret::default();
        hashed_secret_new.0.copy_from_slice(&[1; 32]);
        assert_ok!(AcuityAtomicSwap::lock_buy(Origin::signed(B), AcuityHashedSecret::default(), AcuityChainId::default(), AcuityAdapterId::default(), AcuityOrderId::default(), A, now + 1000, 50, AcuityForeignAddress::default()));
        assert_ok!(AcuityAtomicSwap::lock_buy(Origin::signed(B), hashed_secret_new, AcuityChainId::default(), AcuityAdapterId::default(), AcuityOrderId::default(), A, now + 1000, 50, AcuityForeignAddress::default()));
	});
}

#[test]
fn lock_buy_fail_already_in_use() {
    new_test_ext().execute_with(|| {
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::lock_buy(Origin::signed(B), AcuityHashedSecret::default(), AcuityChainId::default(), AcuityAdapterId::default(), AcuityOrderId::default(), A, now + 1000, 50, AcuityForeignAddress::default()));
		assert_noop!(
            AcuityAtomicSwap::lock_buy(Origin::signed(B), AcuityHashedSecret::default(), AcuityChainId::default(), AcuityAdapterId::default(), AcuityOrderId::default(), A, now + 1000, 50, AcuityForeignAddress::default()),
			Error::<Test>::HashedSecretAlreadyInUse
		);
	});
}

#[test]
fn lock_buy() {
	new_test_ext().execute_with(|| {
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::lock_buy(Origin::signed(B), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityOrderId::default(), A, now + 1000, 50, AcuityForeignAddress::default()));

        let lock = AcuityAtomicSwap::buy_lock(B, hashed_secret).unwrap();
        assert_eq!(lock.seller, A);
        assert_eq!(lock.value, 50);
        assert_eq!(lock.timeout, now + 1000);
	});
}

#[test]
fn unlock_buy_control_timed_out() {
    new_test_ext().execute_with(|| {
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::lock_buy(Origin::signed(B), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityOrderId::default(), A, now + 1000, 50, AcuityForeignAddress::default()));
        assert_ok!(AcuityAtomicSwap::unlock_buy(Origin::signed(A), B, secret));
	});
}

#[test]
fn unlock_buy_fail_timed_out() {
    new_test_ext().execute_with(|| {
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::lock_buy(Origin::signed(B), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityOrderId::default(), A, now, 50, AcuityForeignAddress::default()));
		assert_noop!(
            AcuityAtomicSwap::unlock_buy(Origin::signed(A), B, secret),
			Error::<Test>::LockTimedOut
		);
	});
}

#[test]
fn unlock_buy() {
	new_test_ext().execute_with(|| {
        let mut secret = AcuitySecret::default();
        secret.0.copy_from_slice(&hex::decode("4b1694df15172648181bcb37868b25d3bd9ff95d0f10ec150f783802a81a07fb").unwrap());
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&hex::decode("094cd46013683e3929f474bf04e9ff626a6d7332c195dfe014e4b4a3fbb3ea54").unwrap());
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::lock_buy(Origin::signed(B), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityOrderId::default(), A, now + 1000, 50, AcuityForeignAddress::default()));

        assert_eq!(Balances::free_balance(A), 100);
        assert_eq!(Balances::free_balance(B), 150);
        assert_eq!(Balances::free_balance(AcuityAtomicSwap::fund_account_id()), 50);

        let lock = AcuityAtomicSwap::buy_lock(B, hashed_secret).unwrap();
        assert_eq!(lock.seller, A);
        assert_eq!(lock.value, 50);
        assert_eq!(lock.timeout, now + 1000);

        assert_ok!(AcuityAtomicSwap::unlock_buy(Origin::signed(A), B, secret));

        assert_eq!(Balances::free_balance(A), 150);
        assert_eq!(Balances::free_balance(B), 150);
        assert_eq!(Balances::free_balance(AcuityAtomicSwap::fund_account_id()), 0);

        assert_eq!(AcuityAtomicSwap::buy_lock(B, hashed_secret), None);
	});
}

#[test]
fn timeout_buy_control_not_timed_out() {
    new_test_ext().execute_with(|| {
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::lock_buy(Origin::signed(B), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityOrderId::default(), A, now, 50, AcuityForeignAddress::default()));
        assert_ok!(AcuityAtomicSwap::timeout_buy(Origin::signed(B), secret));
	});
}

#[test]
fn timeout_buy_fail_not_timed_out() {
    new_test_ext().execute_with(|| {
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::lock_buy(Origin::signed(B), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityOrderId::default(), A, now + 1000, 50, AcuityForeignAddress::default()));
		assert_noop!(
            AcuityAtomicSwap::timeout_buy(Origin::signed(B), secret),
			Error::<Test>::LockNotTimedOut
		);
	});
}

#[test]
fn timeout_buy() {
	new_test_ext().execute_with(|| {
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::lock_buy(Origin::signed(B), hashed_secret, AcuityChainId::default(), AcuityAdapterId::default(), AcuityOrderId::default(), A, now, 50, AcuityForeignAddress::default()));

        assert_eq!(Balances::free_balance(A), 100);
        assert_eq!(Balances::free_balance(B), 150);
        assert_eq!(Balances::free_balance(AcuityAtomicSwap::fund_account_id()), 50);

        let lock = AcuityAtomicSwap::buy_lock(B, hashed_secret).unwrap();
        assert_eq!(lock.seller, A);
        assert_eq!(lock.value, 50);
        assert_eq!(lock.timeout, now);

        assert_ok!(AcuityAtomicSwap::timeout_buy(Origin::signed(B), secret));

        assert_eq!(Balances::free_balance(A), 100);
        assert_eq!(Balances::free_balance(B), 200);
        assert_eq!(Balances::free_balance(AcuityAtomicSwap::fund_account_id()), 0);

        assert_eq!(AcuityAtomicSwap::buy_lock(B, hashed_secret), None);
	});
}
*/
