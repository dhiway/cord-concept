#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::result::Result;
// use error;
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, sp_runtime::RuntimeDebug,
	sp_std::prelude::*, traits::EnsureOrigin,
};
use frame_system::{self as system, ensure_signed};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// Custom types
// pub type ProductId = Vec<u8>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Schema<AccountId, Hash> {
	owner: AccountId,
	hash: Hash,
    registered: Moment,
}

/// The Schema trait
pub trait Trait: frame_system::Trait + timestamp::Trait {
	/// Schema specific event type
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type CreateRoleOrigin: EnsureOrigin<Self::Origin>;
}

decl_storage! {
	trait Store for Module<T: Trait> as SchemaRegistry {
		pub Schemas get(fn schemas):map hasher(opaque_blake2_256) T::Hash => Option<Schema<T::AccountId, T::Moment>>;
		pub SchemasOfOrganization get(fn schemas_of_org):map hasher(blake2_128_concat) T::AccountId => Option<T::Hash>;
        pub OwnerOfSchema get(fn owner_of_schema): map hasher(blake2_128_concat) T::Hash => Option<T::AccountId>;
	}
}

decl_event!(
	pub enum Event<T> where <T as frame_system::Trait>::AccountId, <T as frame_system::Trait>::Hash {
		SchemaRegistered(AccountId, Hash),
	}
);

//TBD - add more
decl_error! {
    pub enum Error for Module<T: Trait> {
        SchemaNotFound,
        SchemaExists,
    }
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

		#[weight = 10]

		/// Adds a CTYPE on chain, where
		/// origin - the origin of the transaction
		/// hash - hash of the CTYPE of the claim
		pub fn add(origin, hash: T::Hash) -> dispatch::DispatchResult {
			T::CreateRoleOrigin::ensure_origin(origin.clone())?;

			// origin of the transaction needs to be a signed sender account
            let who = ensure_signed(origin)?;

			// Check product doesn't exist yet (1 DB read)
			Self::validate_new_schema(&hash)?;
			
			// if <Schemas<T>>::contains_key(hash) {
			// 	return Error::<T>::ProductIdExists;
			// }

		// Create a schema instance
			let schema = Self::new_schema()
				.owned_by(owner.clone())
				.schema_hash(hash)
				.registered_on(<timestamp::Module<T>>::now())
				.build();

			// Add schema & ownerOf (3 DB writes)
			<Schemas<T>>::insert(&owner, schema);
			<SchemasOfOrganization<T>>::append(&owner, &hash);
			<OwnerOfSchema<T>>::insert(&owner);

			// <Schemas<T>>::insert(hash, sender.clone());
			// deposit event that the CTYPE has been added
			Self::deposit_event(RawEvent::SchemaCreated(who, hash));
			Ok(())
		}
	}
}


/// Implementation of further module constants and functions for Schemas
impl<T: Trait> Module<T> {

	fn new_schema() -> SchemaBuilder<T::AccountId, T::Hash> {
		SchemaBuilder::<T::AccountId, T::Hash>::default()
	}

	pub fn validate_new_schema(hash: &[u8]) -> Result<(), Error<T>> {
        // Product existence check
        ensure!(
            !<Schemas<T>>::contains_key(hash),
            Error::<T>::SchemaExists
        );
        Ok(())
    }
}

#[derive(Default)]
pub struct SchemaBuilder<AccountId, Hash>
where
    AccountId: Default,
    Moment: Default,
{
	owner: AccountId,
	hash: Hash,
    registered: Moment,
}

impl<AccountId, Hash> SchemaBuilder<AccountId, Hash>
where
    AccountId: Default,
    Moment: Default,
{
	pub fn owned_by(mut self, owner: AccountId) -> Self {
        self.owner = owner;
        self
    }

	pub fn schema_hash(mut self, hash: Hash) -> Self {
        self.hash = hash;
        self
    }

    pub fn registered_on(mut self, registered: Moment) -> Self {
        self.registered = registered;
        self
    }

    pub fn build(self) -> Schema<AccountId, Hash> {
        Product::<AccountId, Hash> {
            owner: self.owner,
            hash: self.hash,
            registered: self.registered,
        }
    }
}