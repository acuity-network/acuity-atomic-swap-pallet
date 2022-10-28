use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use sp_io::hashing::{keccak_256};
use super::*;


const A: u64 = 1;
const B: u64 = 2;

#[test]
fn lock_buy_control_lock_zero_value() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
		let value = 50;
        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, now + 1000, value, AcuityAssetId::default(), 5));
	});
}

#[test]
fn lock_buy_fail_lock_zero_value() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
		let value = 0;
		assert_noop!(
            AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, now + 1000, value, AcuityAssetId::default(), 5),
			Error::<Test>::ZeroValue
		);
	});
}

#[test]
fn lock_buy_control_lock_already_exists() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, now + 1000, 50, AcuityAssetId::default(), 5));
        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, now + 1001, 50, AcuityAssetId::default(), 5));
	});
}

#[test]
fn lock_buy_fail_lock_already_exists() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, now + 1000, 50, AcuityAssetId::default(), 5));
		assert_noop!(
            AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, now + 1000, 50, AcuityAssetId::default(), 5),
			Error::<Test>::LockAlreadyExists
		);
	});
}

#[test]
fn lock_buy() {
	new_test_ext().execute_with(|| {
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get() + 1000;
		let value = 50;
        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, timeout, value, AcuityAssetId::default(), 5));

        let lock_id = AcuityAtomicSwap::get_lock_id(B, A, hashed_secret, timeout);
		assert_eq!(AcuityAtomicSwap::lock_id_value(lock_id).unwrap(), value);
	});
}

#[test]
fn lock_sell_control_lock_zero_value() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
		let value = 50;
        assert_ok!(AcuityAtomicSwap::lock_sell(RuntimeOrigin::signed(B), A, hashed_secret, now + 1000, value, AcuityAssetId::default(), AcuityLockId::default()));
	});
}

#[test]
fn lock_sell_fail_lock_zero_value() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
		let value = 0;
		assert_noop!(
            AcuityAtomicSwap::lock_sell(RuntimeOrigin::signed(B), A, hashed_secret, now + 1000, value, AcuityAssetId::default(), AcuityLockId::default()),
			Error::<Test>::ZeroValue
		);
	});
}

#[test]
fn lock_sell_control_lock_already_exists() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::lock_sell(RuntimeOrigin::signed(B), A, hashed_secret, now + 1000, 50, AcuityAssetId::default(), AcuityLockId::default()));
        assert_ok!(AcuityAtomicSwap::lock_sell(RuntimeOrigin::signed(B), A, hashed_secret, now + 1001, 50, AcuityAssetId::default(), AcuityLockId::default()));
	});
}

#[test]
fn lock_sell_fail_lock_already_exists() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let now = <pallet_timestamp::Pallet<Test>>::get();
        assert_ok!(AcuityAtomicSwap::lock_sell(RuntimeOrigin::signed(B), A, hashed_secret, now + 1000, 50, AcuityAssetId::default(), AcuityLockId::default()));
		assert_noop!(
            AcuityAtomicSwap::lock_sell(RuntimeOrigin::signed(B), A, hashed_secret, now + 1000, 50, AcuityAssetId::default(), AcuityLockId::default()),
			Error::<Test>::LockAlreadyExists
		);
	});
}

#[test]
fn lock_sell() {
	new_test_ext().execute_with(|| {
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get() + 1000;
		let value = 50;
        assert_ok!(AcuityAtomicSwap::lock_sell(RuntimeOrigin::signed(B), A, hashed_secret, timeout, value, AcuityAssetId::default(), AcuityLockId::default()));

        let lock_id = AcuityAtomicSwap::get_lock_id(B, A, hashed_secret, timeout);
		assert_eq!(AcuityAtomicSwap::lock_id_value(lock_id).unwrap(), value);
	});
}

#[test]
fn decline_control_not_exist() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get() + 1000;
		let value = 50;

        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, timeout, value, AcuityAssetId::default(), 5));
		assert_ok!(AcuityAtomicSwap::decline(RuntimeOrigin::signed(A), B, hashed_secret, timeout));
	});
}

#[test]
fn decline_fail_not_exist() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get() + 1000;

		assert_noop!(
            AcuityAtomicSwap::decline(RuntimeOrigin::signed(A), B, hashed_secret, timeout),
			Error::<Test>::LockDoesNotExist
		);
	});
}

#[test]
fn decline() {
	new_test_ext().execute_with(|| {
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get() + 1000;
		let value = 50;
		let lock_id = AcuityAtomicSwap::get_lock_id(B, A, hashed_secret, timeout);

        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, timeout, value, AcuityAssetId::default(), 5));
		assert_eq!(AcuityAtomicSwap::lock_id_value(lock_id), Some(value));

		assert_ok!(AcuityAtomicSwap::decline(RuntimeOrigin::signed(A), B, hashed_secret, timeout));
		assert_eq!(AcuityAtomicSwap::lock_id_value(lock_id), None);
	});
}

#[test]
fn unlock_control_timed_out() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get() + 1000;
		let value = 50;

        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, timeout, value, AcuityAssetId::default(), 5));
		assert_ok!(AcuityAtomicSwap::unlock(RuntimeOrigin::signed(A), B, secret, timeout));
	});
}

#[test]
fn unlock_fail_timed_out() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get();
		let value = 50;

        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, timeout, value, AcuityAssetId::default(), 5));
		assert_noop!(
            AcuityAtomicSwap::unlock(RuntimeOrigin::signed(A), B, secret, timeout),
			Error::<Test>::LockTimedOut
		);
	});
}

#[test]
fn unlock_control_not_exist() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get() + 1000;
		let value = 50;

        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, timeout, value, AcuityAssetId::default(), 5));
		assert_ok!(AcuityAtomicSwap::unlock(RuntimeOrigin::signed(A), B, secret, timeout));
	});
}

#[test]
fn unlock_fail_not_exist() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get() + 1000;

		assert_noop!(
            AcuityAtomicSwap::unlock(RuntimeOrigin::signed(A), B, secret, timeout),
			Error::<Test>::LockDoesNotExist
		);
	});
}

#[test]
fn unlock() {
	new_test_ext().execute_with(|| {
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get() + 1000;
		let value = 50;
		let lock_id = AcuityAtomicSwap::get_lock_id(B, A, hashed_secret, timeout);

        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, timeout, value, AcuityAssetId::default(), 5));
		assert_eq!(AcuityAtomicSwap::lock_id_value(lock_id), Some(value));

		assert_ok!(AcuityAtomicSwap::unlock(RuntimeOrigin::signed(A), B, secret, timeout));
		assert_eq!(AcuityAtomicSwap::lock_id_value(lock_id), None);
	});
}

#[test]
fn retrieve_control_not_timed_out() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get();
		let value = 50;

        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, timeout, value, AcuityAssetId::default(), 5));
		assert_ok!(AcuityAtomicSwap::retrieve(RuntimeOrigin::signed(B), A, hashed_secret, timeout));
	});
}

#[test]
fn retrieve_fail_timed_out() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get() + 1000;
		let value = 50;

        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, timeout, value, AcuityAssetId::default(), 5));

		assert_noop!(
            AcuityAtomicSwap::retrieve(RuntimeOrigin::signed(B), A, hashed_secret, timeout),
			Error::<Test>::LockNotTimedOut
		);
	});
}

#[test]
fn retrieve_control_not_exist() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get();
		let value = 50;

        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, timeout, value, AcuityAssetId::default(), 5));
		assert_ok!(AcuityAtomicSwap::retrieve(RuntimeOrigin::signed(B), A, hashed_secret, timeout));
	});
}

#[test]
fn retrieve_fail_not_exist() {
    new_test_ext().execute_with(|| {
		let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get();

		assert_noop!(
            AcuityAtomicSwap::retrieve(RuntimeOrigin::signed(B), A, hashed_secret, timeout),
			Error::<Test>::LockDoesNotExist
		);
	});
}

#[test]
fn retrieve() {
	new_test_ext().execute_with(|| {
        let secret = AcuitySecret::default();
        let mut hashed_secret = AcuityHashedSecret::default();
        hashed_secret.0.copy_from_slice(&keccak_256(&secret.encode()));
        let timeout = <pallet_timestamp::Pallet<Test>>::get();
		let value = 50;
		let lock_id = AcuityAtomicSwap::get_lock_id(B, A, hashed_secret, timeout);

        assert_ok!(AcuityAtomicSwap::lock_buy(RuntimeOrigin::signed(B), A, hashed_secret, timeout, value, AcuityAssetId::default(), 5));
		assert_eq!(AcuityAtomicSwap::lock_id_value(lock_id), Some(value));

		assert_ok!(AcuityAtomicSwap::retrieve(RuntimeOrigin::signed(B), A, hashed_secret, timeout));
		assert_eq!(AcuityAtomicSwap::lock_id_value(lock_id), None);
	});
}
