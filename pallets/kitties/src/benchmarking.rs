//! Benchmarking setup for pallet-kitties

use super::*;

#[allow(unused)]
use crate::Pallet as KittiesModule;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	create_kitty {
		let price = 100;
		let caller: T::AccountId = whitelisted_caller();
	}: create_kitty(RawOrigin::Signed(caller), price)

	verify {
		assert_eq!(KittyQuantity::<T>::get(), 1);
	}

	change_owner {
		let alice: T::AccountId = whitelisted_caller();
		let bob: T::AccountId = account("bob", 0, 0);
		let price = 100;
		KittiesModule::<T>::create_kitty(RawOrigin::Signed(alice.clone()).into(), price)?;

		let alice_kitty_dna = KittyOwner::<T>::get(&alice).ok_or(Error::<T>::NoneValue).unwrap()[0];


	}: change_owner(RawOrigin::Signed(alice.clone()), alice_kitty_dna, bob.clone())

	verify {
		assert!(KittyOwner::<T>::get(&bob).is_some());
	}

	impl_benchmark_test_suite!(KittiesModule, crate::mock::new_test_ext(), crate::mock::Test);
}
