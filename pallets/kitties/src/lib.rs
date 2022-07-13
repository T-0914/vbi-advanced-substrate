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
use frame_support::inherent::Vec;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;

	pub type DNA = Vec<u8>;

	#[derive(TypeInfo, Encode, Decode, Default)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		dna: DNA,
		owner: T::AccountId,
		price: u32,
		gender: Gender,
	}

	#[derive(TypeInfo, Encode, Decode, Debug)]
	pub enum Gender {
		Male,
		Female,
	}

	impl Default for Gender {
		fn default() -> Self {
			Self::Male
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		// #[pallet::constant]
		// type MaxLength: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn quantity)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type KittyQuantity<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, DNA, Kitty<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kittyowner)]
	pub type KittyOwner<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<DNA>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(DNA, T::AccountId),
		KittyChangeOwner(DNA, T::AccountId, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_kitty(origin: OriginFor<T>, dna: DNA, price: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let owner = ensure_signed(origin)?;

			// Create a new kitty
			let gender = Self::gen_gender(dna.clone())?;
			let kitty = Kitty::<T> { dna: dna.clone(), owner: owner.clone(), price, gender };
			Kitties::<T>::insert(&dna, kitty);

			// Update the current quantity of kitty
			let mut current_quantity = <KittyQuantity<T>>::get();
			current_quantity += 1;
			<KittyQuantity<T>>::put(current_quantity);

			// Update owner's kitties
			if <KittyOwner<T>>::contains_key(&owner) {
				<KittyOwner<T>>::mutate(&owner, |dnas| dnas.push(dna.clone()));
			} else {
				let mut dnas = Vec::new();
				dnas.push(dna.clone());
				<KittyOwner<T>>::insert(&owner, dnas);
			}

			// Emit an event.
			Self::deposit_event(Event::KittyCreated(dna.clone(), owner));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn change_owner(
			origin: OriginFor<T>,
			dna: DNA,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			// Update the new owner in Kitties storage
			let kitty_by_dna = <Kitties<T>>::get(&dna);
			ensure!(kitty_by_dna.is_some(), Error::<T>::NoneValue);

			if let Some(mut kitty) = kitty_by_dna {
				kitty.owner = new_owner.clone();
				<Kitties<T>>::insert(&dna, kitty);
			}

			
			// Update KittyOwner storage
			// Remove the kitty no longer belong to the old owner
			let kitties_except_a_specific = <KittyOwner<T>>::get(&owner)
				.into_iter()
				.filter(|_dna| *_dna != dna.clone())
				.collect::<Vec<DNA>>();
			<KittyOwner<T>>::insert(&owner, kitties_except_a_specific);

			// Add a new kitty to the new owner
			<KittyOwner<T>>::mutate(&new_owner, |dnas| dnas.push(dna.clone()));

			// Emit an event
			Self::deposit_event(Event::KittyChangeOwner(
				dna.clone(),
				owner.clone(),
				new_owner.clone(),
			));

			Ok(())
		}
	}
}

// helper function
impl<T: Config> Pallet<T> {
	fn gen_gender(dna: DNA) -> Result<Gender, Error<T>> {
		let mut gender = Gender::Male;
		if dna.len() % 2 == 0 {
			gender = Gender::Female;
		}
		Ok(gender)
	}
}
