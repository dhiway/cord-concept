// Tests to be written here

use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, dispatch};
use sp_core::H256;

pub fn store_test_credential<T: Trait>(
    id: CredId,
    owner: T::AccountId,
    hash: T::Hash,
    registered: T::Moment,
) {
    Credentials::<T>::insert(
        id.clone(),
        Credential {
            id,
            owner,
            hash,
            registered,
            props: None,
        },
    );
}

const TEST_ORGANIZATION: &str = "Dhiway Test";
const TEST_SENDER: &str = "Ashok";
const TEST_CRED_ID: &str = "00012345600012";
const TEST_CRED_SUBJ: &str = "Test Event Completion for Dhiway";

#[test]
fn create_product_without_props() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let id = TEST_CRED_ID.as_bytes().to_owned();
        let owner = account_key(TEST_ORGANIZATION);
        let hash = H256::from_low_u64_be(1);
        let now = 42;
        Timestamp::set_timestamp(now);

        let result = CredentialRegistry::register_credential(
            Origin::signed(sender),
            id.clone(),
            owner.clone(),
            hash.clone(),
            None,
        );

        assert_ok!(result);

        assert_eq!(
            CredentialRegistry::cred_by_id(&id),
            Some(Credential {
                id: id.clone(),
                owner: owner,
                hash: hash,
                registered: now,
                props: None
            })
        );

        assert_eq!(<CredsOfOrganization<Test>>::get(owner), vec![id.clone()]);
        assert_eq!(CredentialRegistry::issuer_of_cred(&id), Some(owner));
        assert_eq!(<CredByHash<Test>>::get(hash), vec![id.clone()]);

        // Event is raised
        assert!(System::events().iter().any(|er| er.event
            == TestEvent::credential_registry(RawEvent::CredentialRegistered(
                sender,
                id.clone(),
                hash.clone(),
            ))));
    });
}

#[test]
fn create_cred_with_valid_props() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let id = TEST_CRED_ID.as_bytes().to_owned();
        let owner = account_key(TEST_ORGANIZATION);
        let hash = H256::from_low_u64_be(1);
        let now = 42;
        Timestamp::set_timestamp(now);

        let result = CredentialRegistry::register_credential(
            Origin::signed(sender),
            id.clone(),
            owner.clone(),
            hash.clone(),
            Some(vec![CredProperty::new(
                &TEST_CRED_SUBJ.as_bytes().to_owned(),
            )]),
        );

        assert_ok!(result);

        assert_eq!(
            CredentialRegistry::cred_by_id(&id),
            Some(Credential {
                id: id.clone(),
                owner: owner,
                hash: hash.clone(),
                registered: now,
                props: Some(vec![CredProperty::new(
                    &TEST_CRED_SUBJ.as_bytes().to_owned()
                )]),
            })
        );

        assert_eq!(<CredsOfOrganization<Test>>::get(owner), vec![id.clone()]);
        assert_eq!(CredentialRegistry::issuer_of_cred(&id), Some(owner));
        assert_eq!(<CredByHash<Test>>::get(hash), vec![id.clone()]);

        // Event is raised
        assert!(System::events().iter().any(|er| er.event
            == TestEvent::credential_registry(RawEvent::CredentialRegistered(
                sender,
                id.clone(),
                hash.clone(),
            ))));
    });
}

#[test]
fn create_cred_with_invalid_sender() {
    new_test_ext().execute_with(|| {
        let hash = H256::from_low_u64_be(1);
        assert_noop!(
            CredentialRegistry::register_credential(
                Origin::none(),
                vec!(),
                account_key(TEST_ORGANIZATION),
                hash.clone(),
                None
            ),
            dispatch::DispatchError::BadOrigin
        );
    });
}

#[test]
fn create_cred_with_missing_id() {
    new_test_ext().execute_with(|| {
        let hash = H256::from_low_u64_be(1);
        assert_noop!(
            CredentialRegistry::register_credential(
                Origin::signed(account_key(TEST_SENDER)),
                vec!(),
                account_key(TEST_ORGANIZATION),
                hash.clone(),
                None
            ),
            Error::<Test>::CredIdMissing
        );
    });
}

#[test]
fn create_cred_with_long_id() {
    new_test_ext().execute_with(|| {
        let hash = H256::from_low_u64_be(1);
        assert_noop!(
            CredentialRegistry::register_credential(
                Origin::signed(account_key(TEST_SENDER)),
                b"ajasljdfalsjfasdfjasfasdlfjasdflkajdsflkajdsfalksjdfalksdjfadfasdf".to_vec(),
                account_key(TEST_ORGANIZATION),
                hash.clone(),
                None
            ),
            Error::<Test>::CredIdTooLong
        );
    })
}

#[test]
fn create_cred_with_existing_id() {
    new_test_ext().execute_with(|| {
        let existing_cred = TEST_CRED_ID.as_bytes().to_owned();
        let hash = H256::from_low_u64_be(1);
        let now = 42;

        store_test_credential::<Test>(
            existing_cred.clone(),
            account_key(TEST_ORGANIZATION),
            hash.clone(),
            now,
        );

        assert_noop!(
            CredentialRegistry::register_credential(
                Origin::signed(account_key(TEST_SENDER)),
                existing_cred,
                account_key(TEST_ORGANIZATION),
                hash.clone(),
                None
            ),
            Error::<Test>::CredIdExists
        );
    })
}

#[test]
fn create_cred_with_too_many_props() {
    new_test_ext().execute_with(|| {
        let hash = H256::from_low_u64_be(1);
        assert_noop!(
            CredentialRegistry::register_credential(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_CRED_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                hash.clone(),
                Some(vec![
                    CredProperty::new(b"Subject to Ashok Kumar"),
                    CredProperty::new(b"Subject to John Doe"),
                ])
            ),
            Error::<Test>::CredTooManyProps
        );
    })
}

#[test]
fn create_cred_with_invalid_subject() {
    new_test_ext().execute_with(|| {
        let hash = H256::from_low_u64_be(1);
        assert_noop!(
            CredentialRegistry::register_credential(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_CRED_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                hash.clone(),
                Some(vec![CredProperty::new(
                    b"This is a long event name where I can't get it for a particular person"
                )])
            ),
            Error::<Test>::CredInvalidSubject
        );
    })
}
