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
use frame_support::sp_runtime::ArithmeticError;
use frame_support::traits::Time;
use frame_support::{sp_runtime::app_crypto::sp_core::H256, traits::Randomness};
use frame_system::pallet_prelude::*;

pub type CreatedDate<T> = <<T as Config>::CreatedDate as frame_support::traits::Time>::Moment;
pub type DnaHashType = H256;

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;

	#[derive(TypeInfo, Encode, Decode, Default)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		dna: DnaHashType,
		owner: T::AccountId,
		price: u32,
		gender: Gender,
		created_date: CreatedDate<T>,
	}

	#[derive(TypeInfo, Encode, Decode, Debug, Default)]

	pub enum Gender {
		#[default]
		Male,
		Female,
	}

	#[derive(Default)]

	pub enum StorageUpdateType {
		#[default]
		CreateKitty,
		ChangeOwner,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type CreatedDate: Time;

		type RandomnessSource: Randomness<DnaHashType, Self::BlockNumber>;

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
	pub type Kitties<T: Config> =
		StorageMap<_, Blake2_128Concat, DnaHashType, Kitty<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<DnaHashType, T::KittyLimit>,
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(DnaHashType, T::AccountId),
		KittyChangeOwner(DnaHashType, T::AccountId, T::AccountId),
		DnaGenerated(DnaHashType),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		OwnerNotFound,
		KittyNotFound,
		KittyAlreadyExist,
		DuplicatedOwner,
		KittyLimitReached,
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
			let gender = Self::gen_gender(&dna)?;

			// Create a new kitty
			let kitty = Kitty::<T> {
				dna: dna.clone(),
				owner: owner.clone(),
				price,
				gender,
				created_date: T::CreatedDate::now(),
			};

			// Update the current quantity of kitty
			let current_quantity = <KittyQuantity<T>>::get();
			let new_quantity = current_quantity.checked_add(1).ok_or(ArithmeticError::Overflow)?;
			<KittyQuantity<T>>::put(new_quantity);

			Kitties::<T>::insert(&dna, kitty);

			// Update owner's kitties
			Self::update_kitties_of_owner(
				owner.clone(),
				owner.clone(),
				dna.clone(),
				StorageUpdateType::CreateKitty,
			)?;
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn change_owner(
			origin: OriginFor<T>,
			dna: DnaHashType,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			ensure!(owner != new_owner, Error::<T>::DuplicatedOwner);

			// Update the new owner in Kitties storage
			let mut kitty_by_dna = <Kitties<T>>::get(&dna).ok_or(Error::<T>::KittyNotFound)?;
			kitty_by_dna.owner = new_owner.clone();
			<Kitties<T>>::insert(&dna, kitty_by_dna);

			// Update KittyOwner storage

			// Remove the kitty of the old owner
			let mut kitties_of_owner =
				<KittyOwner<T>>::get(&owner).ok_or(<Error<T>>::OwnerNotFound)?;
			kitties_of_owner.retain(|_dna| _dna != &dna);
			<KittyOwner<T>>::insert(&owner, kitties_of_owner);

			// Add a new kitty to the new owner
			Self::update_kitties_of_owner(
				owner.clone(),
				new_owner.clone(),
				dna.clone(),
				StorageUpdateType::ChangeOwner,
			)?;
			Ok(())
		}
	}
}

// helper function
impl<T: Config> Pallet<T> {
	fn gen_gender(dna: &DnaHashType) -> Result<Gender, Error<T>> {
		let dna_vec = dna.as_bytes().to_vec();
		let gender = dna_vec.get(1);

		match gender {
			Some(_gender) => {
				if *_gender % 2 == 0 {
					Ok(Gender::Male)
				} else {
					Ok(Gender::Female)
				}
			},
			None => Err(<Error<T>>::NoneValue),
		}
	}

	fn encode_and_update_nonce() -> Vec<u8> {
		let nonce = Nonce::<T>::get();
		Nonce::<T>::put(nonce.wrapping_add(1));
		nonce.encode()
	}

	fn gen_dna() -> Result<DnaHashType, Error<T>> {
		let dna_nonce = Self::encode_and_update_nonce();

		let (dna_random_seed, _) = T::RandomnessSource::random_seed();
		let (dna, _) = T::RandomnessSource::random(&dna_nonce);

		Self::deposit_event(Event::<T>::DnaGenerated(dna_random_seed));
		Ok(dna)
	}

	fn update_kitties_of_owner(
		who: T::AccountId,
		owner: T::AccountId,
		kitty_dna: DnaHashType,
		update_type: StorageUpdateType,
	) -> Result<(), Error<T>> {
		let event = match update_type {
			StorageUpdateType::CreateKitty => {
				Event::<T>::KittyCreated(kitty_dna.clone(), who.clone())
			},
			StorageUpdateType::ChangeOwner => {
				Event::<T>::KittyChangeOwner(kitty_dna.clone(), who.clone(), owner.clone())
			},
		};
		match <KittyOwner<T>>::try_get(&owner) {
			Ok(mut _kitties) => match _kitties.binary_search(&kitty_dna) {
				Ok(_) => Err(<Error<T>>::KittyAlreadyExist.into()),
				Err(_) => {
					_kitties
						.try_push(kitty_dna.clone())
						.map_err(|_| <Error<T>>::KittyLimitReached)?;

					<KittyOwner<T>>::insert(&owner, _kitties);

					Self::deposit_event(event);
					Ok(().into())
				},
			},
			Err(_) => {
				let mut _kitty_dnas = Vec::new();
				_kitty_dnas.push(kitty_dna.clone());

				let bounded_dnas =
					<BoundedVec<DnaHashType, T::KittyLimit>>::truncate_from(_kitty_dnas);
				<KittyOwner<T>>::insert(&owner, bounded_dnas.clone());

				Self::deposit_event(event);
				Ok(().into())
			},
		}
	}
}
