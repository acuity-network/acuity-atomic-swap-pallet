use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use sp_io::hashing::{blake2_128};



const A: u64 = 1;
const B: u64 = 2;


#[test]
fn add_to_order() {
	new_test_ext().execute_with(|| {
        let price: u128 = 5;
		assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), price, 50));

        assert_eq!(Balances::free_balance(A), 50);
        assert_eq!(Balances::free_balance(AcuityAtomicSwap::fund_account_id()), 50);

        let order_id: [u8; 16] = blake2_128(&[A.to_ne_bytes().to_vec(), price.to_ne_bytes().to_vec()].concat());

        assert_eq!(AcuityAtomicSwap::order_id_value(order_id), 50);

	});
}

#[test]
fn remove_from_order_control_order_to_small() {
	new_test_ext().execute_with(|| {
        let price: u128 = 5;
        assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), price, 50));
        assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), price, 50));
	});
}

#[test]
fn remove_from_order_fail_order_to_small() {
	new_test_ext().execute_with(|| {
        let price: u128 = 5;
        assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), price, 50));
		assert_noop!(
            AcuityAtomicSwap::remove_from_order(Origin::signed(A), price, 51),
			Error::<Test>::OrderTooSmall
		);
	});
}

#[test]
fn remove_from_order() {
	new_test_ext().execute_with(|| {
        let price: u128 = 5;
		assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), price, 50));

        assert_eq!(Balances::free_balance(A), 50);
        assert_eq!(Balances::free_balance(AcuityAtomicSwap::fund_account_id()), 50);

        let order_id: [u8; 16] = blake2_128(&[A.to_ne_bytes().to_vec(), price.to_ne_bytes().to_vec()].concat());

        assert_eq!(AcuityAtomicSwap::order_id_value(order_id), 50);

        assert_ok!(AcuityAtomicSwap::remove_from_order(Origin::signed(A), price, 50));
        assert_eq!(Balances::free_balance(A), 100);
        assert_eq!(Balances::free_balance(AcuityAtomicSwap::fund_account_id()), 0);
        assert_eq!(AcuityAtomicSwap::order_id_value(order_id), 0);

	});
}

#[test]
fn lock_sell_order_control_too_small() {
    new_test_ext().execute_with(|| {
        let price: u128 = 5;
        assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), price, 50));
        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), price, [0; 32], 0, 50));
	});
}

#[test]
fn lock_sell_order_fail_too_small() {
    new_test_ext().execute_with(|| {
        let price: u128 = 5;
        assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), price, 50));
		assert_noop!(
            AcuityAtomicSwap::lock_sell(Origin::signed(A), price, [0; 32], 0, 60),
			Error::<Test>::OrderTooSmall
		);
	});
}

#[test]
fn lock_sell_order_control_already_in_use() {
    new_test_ext().execute_with(|| {
        let price: u128 = 5;
        assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), price, 50));
        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), price, [0; 32], 0, 10));
        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), price, [1; 32], 0, 10));
	});
}

#[test]
fn lock_sell_order_fail_already_in_use() {
    new_test_ext().execute_with(|| {
        let price: u128 = 5;
        assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), price, 50));
        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), price, [0; 32], 0, 10));
		assert_noop!(
            AcuityAtomicSwap::lock_sell(Origin::signed(A), price, [0; 32], 0, 10),
			Error::<Test>::HashedSecretAlreadyInUse
		);
	});
}

#[test]
fn lock_sell() {
	new_test_ext().execute_with(|| {
        let price: u128 = 5;
		assert_ok!(AcuityAtomicSwap::add_to_order(Origin::signed(A), price, 50));

        assert_eq!(Balances::free_balance(A), 50);
        assert_eq!(Balances::free_balance(AcuityAtomicSwap::fund_account_id()), 50);

        let order_id: [u8; 16] = blake2_128(&[A.to_ne_bytes().to_vec(), price.to_ne_bytes().to_vec()].concat());

        assert_eq!(AcuityAtomicSwap::order_id_value(order_id), 50);

        assert_ok!(AcuityAtomicSwap::lock_sell(Origin::signed(A), price, [0; 32], 0, 10));
        assert_eq!(AcuityAtomicSwap::order_id_value(order_id), 40);

        let lock = AcuityAtomicSwap::sell_lock([0; 32]);
        assert_eq!(lock.order_id, order_id);
        assert_eq!(lock.value, 10);
        assert_eq!(lock.timeout, 0);

	});
}
