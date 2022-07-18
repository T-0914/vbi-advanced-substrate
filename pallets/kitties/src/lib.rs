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
use frame_support::traits::Randomness;
use frame_support::traits::Time;
use frame_system::pallet_prelude::*;

pub type CreatedDate<T> = <<T as Config>::CreatedDate as frame_support::traits::Time>::Moment;
#[frame_support::pallet]
pub mod pallet {
	pub use super::*;

	#[derive(TypeInfo, Encode, Decode, Default)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		dna: T::Hash,
		owner: T::AccountId,
		price: u32,
		gender: Gender,
		created_date: CreatedDate<T>,
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

		type CreatedDate: Time;

		type RandomnessSource: Randomness<Self::Hash, Self::BlockNumber>;

		#[pallet::constant]
		type KittyLimit: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn nonce)]
	pub(super) type Nonce<T: Config> = StorageValue<_, u32, ValueQuery>;

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn quantity)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type KittyQuantity<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Kitty<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::KittyLimit>,
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::Hash, T::AccountId),
		KittyChangeOwner(T::Hash, T::AccountId, T::AccountId),
		DnaGenerated(T::Hash),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		DuplicatedOwner,
		ExceedLimit,
		MoveValueNotExist,
		MoveValueAlreadyExist,
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
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_kitty(origin: OriginFor<T>, price: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let owner = ensure_signed(origin)?;

			let dna = Self::gen_dna()?;
			let gender = Self::gen_gender()?;

			// Create a new kitty
			let kitty = Kitty::<T> {
				dna: dna.clone(),
				owner: owner.clone(),
				price,
				gender,
				created_date: T::CreatedDate::now(),
			};
			Kitties::<T>::insert(&dna, kitty);

			// Update the current quantity of kitty
			let mut current_quantity = <KittyQuantity<T>>::get();
			current_quantity += 1;
			<KittyQuantity<T>>::put(current_quantity);

			// Update owner's kitties
			if <KittyOwner<T>>::contains_key(&owner) {
				<KittyOwner<T>>::try_mutate(&owner, |dnas| match dnas {
					Some(_dnas) => _dnas.try_push(dna.clone()).map_err(|_| Error::<T>::ExceedLimit),
					_ => Err(Error::<T>::NoneValue),
				})?;
			} else {
				let mut dnas = Vec::new();
				dnas.push(dna.clone());
				let bounded_dnas = <BoundedVec<T::Hash, T::KittyLimit>>::truncate_from(dnas);
				<KittyOwner<T>>::insert(&owner, bounded_dnas);
			}

			// Emit an event.
			Self::deposit_event(Event::KittyCreated(dna.clone(), owner));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn change_owner(
			origin: OriginFor<T>,
			dna: T::Hash,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			ensure!(owner != new_owner, Error::<T>::DuplicatedOwner);

			// Update the new owner in Kitties storage
			let kitty_by_dna = <Kitties<T>>::get(&dna);
			ensure!(kitty_by_dna.is_some(), Error::<T>::NoneValue);

			if let Some(mut kitty) = kitty_by_dna {
				kitty.owner = new_owner.clone();
				<Kitties<T>>::insert(&dna, kitty);
			}

			// Update KittyOwner storage
			// Remove the kitty no longer belong to the old owner
			let kitty_owners = Self::kitty_owner(&owner);
			ensure!(kitty_owners.is_some(), Error::<T>::MoveValueNotExist);
			<KittyOwner<T>>::mutate(&owner, |dnas| {
				if let Some(_dnas) = dnas {
					_dnas.retain(|__dna| __dna != &dna);
				}
			});

			// Add a new kitty to the new owner
			if <KittyOwner<T>>::contains_key(&new_owner) {
				<KittyOwner<T>>::mutate(&new_owner, |dnas| {
					if let Some(_dnas) = dnas {
						_dnas
							.try_push(dna.clone())
							.expect("Already full! Can not receive any kitty more!");
					}
				});
			} else {
				let mut _dnas = Vec::new();
				_dnas.push(dna.clone());

				let bounded_dnas = <BoundedVec<T::Hash, T::KittyLimit>>::truncate_from(_dnas);
				<KittyOwner<T>>::insert(&new_owner, bounded_dnas);
			}

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
	fn gen_gender() -> Result<Gender, Error<T>> {
		Ok(Gender::Male)
	}

	fn encode_and_update_nonce() -> Vec<u8> {
		let nonce = Nonce::<T>::get();
		Nonce::<T>::put(nonce.wrapping_add(1));
		nonce.encode()
	}

	fn gen_dna() -> Result<T::Hash, Error<T>> {
		let dna_nonce = Self::encode_and_update_nonce();

		let (dna_random_seed, _) = T::RandomnessSource::random_seed();
		let (dna, _) = T::RandomnessSource::random(&dna_nonce);

		Self::deposit_event(Event::<T>::DnaGenerated(dna_random_seed));
		Ok(dna)
	}
}
