//! # Credential Registry pallet

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
pub const CRED_ID_MAX_LENGTH: usize = 24;
pub const CRED_SUBJ_MAX_LENGTH: usize = 48;
pub const CRED_MAX_PROPS: usize = 1;

// Custom types
pub type CredId = Vec<u8>;
pub type CredSubject = Vec<u8>;


// Credential contains master data (aka class-level) about a credential item.
// This data is typically registered when a credential is signed, and remains static.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Credential<AccountId, Hash, Moment> {
    id: CredId,
    owner: AccountId,
    hash: Hash,
    props: Option<Vec<CredProperty>>,
    registered: Moment,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct CredProperty {
    subject: CredSubject
}

impl CredProperty {
    pub fn new(subject: &[u8]) -> Self {
        Self {
            subject: subject.to_vec(),
        }
    }

    pub fn subject(&self) -> &[u8] {
        self.subject.as_ref()
    }
}

pub trait Trait: frame_system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type CreateRoleOrigin: EnsureOrigin<Self::Origin>;
}

decl_storage! {
    trait Store for Module<T: Trait> as CredentialRegistry {
        pub Credentials get(fn cred_by_id): map hasher(blake2_128_concat) CredId => Option<Credential<T::AccountId, T::Hash, T::Moment>>;
        pub CredsOfOrganization get(fn cred_of_org): map hasher(blake2_128_concat) T::AccountId => Vec<CredId>;
        pub IssuerOfCred get(fn issuer_of_cred): map hasher(blake2_128_concat) CredId => Option<T::AccountId>;
        pub CredByHash get(fn cred_by_hash):map hasher(opaque_blake2_256) T::Hash => Vec<CredId>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Hash = <T as frame_system::Trait>::Hash,
    {
        CredentialRegistered(AccountId, CredId, Hash),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        CredIdMissing,
        CredIdTooLong,
        CredIdExists,
        CredInvalidSubject,
        CredTooManyProps,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        #[weight = 10]
        pub fn register_credential(origin, id: CredId, owner: T::AccountId, hash: T::Hash, 
            props: Option<Vec<CredProperty>>) -> dispatch::DispatchResult {

            T::CreateRoleOrigin::ensure_origin(origin.clone())?;
            let who = ensure_signed(origin)?;
            
            // Validate Cred ID
            Self::validate_cred_id(&id)?;

            // Validate credential props
            Self::validate_cred_props(&props)?;

            // Check credential doesn't exist yet (1 DB read)
            Self::validate_new_credential(&id)?;

            // Create a credential instance
            let credential = Self::new_credential()
                .identified_by(id.clone())
                .owned_by(owner.clone())
                .credential_hash(hash.clone())
                .registered_on(<timestamp::Module<T>>::now())
                .with_props(props)
                .build();

            // Add Cred & ownerOf (4 DB writes)
            <Credentials<T>>::insert(&id, credential);
            <CredsOfOrganization<T>>::append(&owner, &id);
            <IssuerOfCred<T>>::insert(&id, &owner);
            <CredByHash<T>>::append(&hash, &id);
            
            Self::deposit_event(RawEvent::CredentialRegistered(who, id, hash));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    // Helper methods
    fn new_credential() -> CredentialBuilder<T::AccountId, T::Hash, T::Moment> {
        CredentialBuilder::<T::AccountId, T::Hash, T::Moment>::default()
    }

    pub fn validate_cred_id(id: &[u8]) -> Result<(), Error<T>> {
        ensure!(!id.is_empty(), Error::<T>::CredIdMissing);
        ensure!(
            id.len() <= CRED_ID_MAX_LENGTH,
            Error::<T>::CredIdTooLong
        );
        Ok(())
    }

    pub fn validate_new_credential(id: &[u8]) -> Result<(), Error<T>> {
        // TODO - Chnage this to check hash
        ensure!(
            !<Credentials<T>>::contains_key(id),
            Error::<T>::CredIdExists
        );
        Ok(())
    }

    pub fn validate_cred_props(props: &Option<Vec<CredProperty>>) -> Result<(), Error<T>> {
        if let Some(props) = props {
            ensure!(
                props.len() <= CRED_MAX_PROPS,
                Error::<T>::CredTooManyProps,
            );
            for prop in props {
                ensure!(
                    prop.subject().len() <= CRED_SUBJ_MAX_LENGTH,
                    Error::<T>::CredInvalidSubject
                );
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct CredentialBuilder<AccountId, Hash, Moment>
where
    AccountId: Default,
    Hash: Default,
    Moment: Default,
{
    id: CredId,
    owner: AccountId,
    hash: Hash,
    props: Option<Vec<CredProperty>>,
    registered: Moment,
}

impl<AccountId, Hash, Moment> CredentialBuilder<AccountId, Hash, Moment>
where
    AccountId: Default,
    Hash: Default,
    Moment: Default,
{
    pub fn identified_by(mut self, id: CredId) -> Self {
        self.id = id;
        self
    }

    pub fn owned_by(mut self, owner: AccountId) -> Self {
        self.owner = owner;
        self
    }
    pub fn credential_hash(mut self, hash: Hash) -> Self {
        self.hash = hash;
        self
    }

    pub fn with_props(mut self, props: Option<Vec<CredProperty>>) -> Self {
        self.props = props;
        self
    }

    pub fn registered_on(mut self, registered: Moment) -> Self {
        self.registered = registered;
        self
    }

    pub fn build(self) -> Credential<AccountId, Hash, Moment> {
        Credential::<AccountId, Hash, Moment> {
            id: self.id,
            owner: self.owner,
            hash: self.hash,
            props: self.props,
            registered: self.registered,
        }
    }
}
