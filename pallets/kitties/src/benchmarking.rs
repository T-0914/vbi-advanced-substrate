//! Benchmarking setup for pallet-kitties

use super::*;

#[allow(unused)]
use crate::Pallet as Kitties;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	create_kitty {
		let price = 100;
		let caller: T::AccountId = whitelisted_caller();
	}: create_kitty(RawOrigin::Signed(caller), price);
	verify {
		assert_eq!(KittyQuantity::<T>::get(), 1);
	}

	impl_benchmark_test_suite!(Kitties, crate::mock::new_test_ext(), crate::mock::Test);
}
