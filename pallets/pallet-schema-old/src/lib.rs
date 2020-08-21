//! # Shema Registry pallet

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::result::Result;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, sp_runtime::RuntimeDebug,
    sp_std::prelude::*, traits::EnsureOrigin,
};
use frame_system::{self as system, ensure_signed};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// General constraints to limit data size
pub const SCHEMA_ID_MAX_LENGTH: usize = 24;
pub const SCHEMA_NAME_MAX_LENGTH: usize = 24;
pub const SCHEMA_VERSION_MAX_LENGTH: usize = 8;
pub const SCHEMA_DESC_MAX_LENGTH: usize = 256;
pub const SCHEMA_MAX_PROPS: usize = 4;

// Custom types
pub type SchemaId = Vec<u8>;
pub type SchemaName = Vec<u8>;
pub type SchemaVersion = Vec<u8>;
pub type SchemaDesc = Vec<u8>;

// Schema contains master data (aka class-level) about a credential item.
// This data is typically registered once by Dhiway,
// to be shared with other network participants, and remains largely static.
// It can also be used for instance-level (lot) master data.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Schema<AccountId, Hash, Moment> {
    id: SchemaId,
    owner: AccountId,
    hash: Hash,
    version: SchemaVersion,
    props: Option<Vec<SchemaProperty>>,
    registered: Moment,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct SchemaProperty {
    name: SchemaName,
    desc: SchemaDesc,
}

impl SchemaProperty {
    pub fn new(name: &[u8], desc: &[u8]) -> Self {
        Self {
            name: name.to_vec(),
            desc: desc.to_vec(),
        }
    }

    pub fn name(&self) -> &[u8] {
        self.name.as_ref()
    }

    pub fn desc(&self) -> &[u8] {
        self.desc.as_ref()
    }
}

pub trait Trait: system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type CreateRoleOrigin: EnsureOrigin<Self::Origin>;
}

decl_storage! {
    trait Store for Module<T: Trait> as SchemaRegistry {
        pub Schemas get(fn schema_by_id): map hasher(blake2_128_concat) SchemaId => Option<Schema<T::AccountId, T::Hash, T::Moment>>;
        pub SchemasOfOrganization get(fn schemas_of_org): map hasher(blake2_128_concat) T::AccountId => Vec<SchemaId>;
        pub OwnerOfSchema get(fn owner_of): map hasher(blake2_128_concat) SchemaId => Option<T::AccountId>;
        pub SchemaByHash get(fn schema_by_hash):map hasher(opaque_blake2_256) T::Hash => Vec<SchemaId>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Hash = <T as frame_system::Trait>::Hash,
    {
        SchemaRegistered(AccountId, SchemaId, SchemaVersion, Hash),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        SchemaIdMissing,
        SchemaIdTooLong,
        SchemaIdExists,
        SchemaVersionExists,
        SchemaVersionMissing,
        SchemaTooManyProps,
        SchemaInvalidName,
        SchemaInvalidDescription
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        #[weight = 10]
        pub fn register_schema(origin, id: SchemaId, owner: T::AccountId, version: SchemaVersion,
                hash: T::Hash, props: Option<Vec<SchemaProperty>>) -> dispatch::DispatchResult {

            T::CreateRoleOrigin::ensure_origin(origin.clone())?;
            let who = ensure_signed(origin)?;

            // Validate schema ID
            Self::validate_schema_id(&id)?;

            // Validate schema version
            Self::validate_schema_version(&version)?;

            // Validate schema props
            Self::validate_schema_props(&props)?;

            // Check schema doesn't exist yet (1 DB read)
            Self::validate_new_schema(&id)?;
            
            // Create a product instance
            let schema = Self::new_schema()
                .identified_by(id.clone())
                .owned_by(owner.clone())
                .schema_hash(hash.clone())
                .schema_version(version.clone())
                .registered_on(<timestamp::Module<T>>::now())
                .with_props(props)
                .build();

            // Add product & ownerOf (3 DB writes)
            <Schemas<T>>::insert(&id, schema);
            <SchemasOfOrganization<T>>::append(&owner, &id);
            <OwnerOfSchema<T>>::insert(&id, &owner);
            <SchemaByHash<T>>::append(&hash, &id);
            
            Self::deposit_event(RawEvent::SchemaRegistered(who, id, version, hash));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    // Helper methods
    fn new_schema() -> SchemaBuilder<T::AccountId, T::Hash, T::Moment> {
        SchemaBuilder::<T::AccountId, T::Hash, T::Moment>::default()
    }

    pub fn validate_schema_id(id: &[u8]) -> Result<(), Error<T>> {
        ensure!(!id.is_empty(), Error::<T>::SchemaIdMissing);
        ensure!(
            id.len() <= SCHEMA_ID_MAX_LENGTH,
            Error::<T>::SchemaIdTooLong
        );
        Ok(())
    }

    pub fn validate_schema_version(version: &[u8]) -> Result<(), Error<T>> {
        ensure!(!version.is_empty(), Error::<T>::SchemaVersionMissing);
        // ensure!(
        //     !<Schemas<T>>::contains_key(schema::schema_version),
        //     Error::<T>::SchemaVersionExists
        // );
        Ok(())
    }

    pub fn validate_new_schema(id: &[u8]) -> Result<(), Error<T>> {
        // Product existence check
        ensure!(
            !<Schemas<T>>::contains_key(id),
            Error::<T>::SchemaIdExists
        );
        Ok(())
    }

    pub fn validate_schema_props(props: &Option<Vec<SchemaProperty>>) -> Result<(), Error<T>> {
        if let Some(props) = props {
            ensure!(
                props.len() <= SCHEMA_MAX_PROPS,
                Error::<T>::SchemaTooManyProps,
            );
            for prop in props {
                ensure!(
                    prop.name().len() <= SCHEMA_NAME_MAX_LENGTH,
                    Error::<T>::SchemaInvalidName
                );
                ensure!(
                    prop.desc().len() <= SCHEMA_DESC_MAX_LENGTH,
                    Error::<T>::SchemaInvalidDescription
                );
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct SchemaBuilder<AccountId, Hash, Moment>
where
    AccountId: Default,
    Hash: Default,
    Moment: Default,
{
    id: SchemaId,
    owner: AccountId,
    hash: Hash,
    version: SchemaVersion,
    props: Option<Vec<SchemaProperty>>,
    registered: Moment,
}

impl<AccountId, Hash, Moment> SchemaBuilder<AccountId, Hash, Moment>
where
    AccountId: Default,
    Hash: Default,
    Moment: Default,
{
    pub fn identified_by(mut self, id: SchemaId) -> Self {
        self.id = id;
        self
    }

    pub fn owned_by(mut self, owner: AccountId) -> Self {
        self.owner = owner;
        self
    }
    pub fn schema_hash(mut self, hash: Hash) -> Self {
        self.hash = hash;
        self
    }

    pub fn schema_version(mut self, version: SchemaVersion) -> Self {
        self.version = version;
        self
    }

    pub fn with_props(mut self, props: Option<Vec<SchemaProperty>>) -> Self {
        self.props = props;
        self
    }

    pub fn registered_on(mut self, registered: Moment) -> Self {
        self.registered = registered;
        self
    }

    pub fn build(self) -> Schema<AccountId, Hash, Moment> {
        Schema::<AccountId, Hash, Moment> {
            id: self.id,
            owner: self.owner,
            hash: self.hash,
            version: self.version,
            props: self.props,
            registered: self.registered,
        }
    }
}
