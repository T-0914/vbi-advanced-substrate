#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

use frame_support::{dispatch::fmt, inherent::Vec, pallet_prelude::*};
use frame_system::pallet_prelude::*;

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;
	#[derive(TypeInfo, Default, Encode, Decode)]
	#[scale_info(skip_type_params(T))]
	pub struct Students<T: Config> {
		name: Vec<u8>,
		age: u8,
		gender: Gender,
		account: T::AccountId,
	}

	impl<T: Config> fmt::Debug for Students<T> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Students")
			 .field("name", &self.name)
			 .field("age", &self.gender)
			 .field("account", &self.account)
			 .finish()
		}
	}

	pub type Id = u32;

	#[derive(TypeInfo, Encode, Decode, Debug, Clone)]
	pub enum Gender {
		Male,
		Female,
	}

	impl Default for Gender {
		fn default() -> Self {
			Gender::Male
		}
	}
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn student_id)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type StudentId<T> = StorageValue<_, Id, ValueQuery>;

	// key : id
	//value : student
	#[pallet::storage]
	#[pallet::getter(fn student)]
	pub(super) type Student<T: Config> =
		StorageMap<_, Blake2_128Concat, Id, Students<T>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		StudentStored(Vec<u8>, u8),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		TooYoung,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.

	//extrinsic
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_student(origin: OriginFor<T>, name: Vec<u8>, age: u8) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;
			ensure!(age > 20, Error::<T>::TooYoung);
			let gender = Self::gen_gender(name.clone())?;
			let student = Students { name: name.clone(), age, gender: gender.clone(), account: who };
			// let current_id = Self::student_id();
			// let current_id = StudentId::<T>::get();
			let mut current_id = <StudentId<T>>::get();

			log::info!("Current id : {}", current_id);
			log::info!("Genderr : {:?}", gender);
			log::warn!("Student : {:?}", &student);

			// Student::<T>::insert(current_id, student);
			<Student<T>>::insert(current_id, student);
			current_id += 1;
			StudentId::<T>::put(current_id);
			// Emit an event.
			Self::deposit_event(Event::StudentStored(name, age));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}

// helper function

impl<T> Pallet<T> {
	fn gen_gender(name: Vec<u8>) -> Result<Gender, Error<T>> {
		let mut res = Gender::Male;
		if name.len() % 2 == 0 {
			res = Gender::Female;
		}
		Ok(res)
	}
}

// Tóm tắt:
//Custom type: Struct ,Enum
// Sử dụng generic type đối với trait
// helper function
// origin
// một số method cơ bản liên quan tới read/write storage
// giải quuêys một số bug có thể có .
