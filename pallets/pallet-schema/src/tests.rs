// Tests to be written here

use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, dispatch};
use sp_core::{H256};

pub fn store_test_schema<T: Trait>(id: SchemaId, owner: T::AccountId, hash: T::Hash, registered: T::Moment) {
    Schemas::<T>::insert(
        id.clone(),
        Schema {
            id,
            owner,
            hash,
            registered,
            props: None,
        },
    );
}

const TEST_ORGANIZATION: &str = "Dhiway Test";
const TEST_SENDER: &str = "Alice";
const TEST_SCHEMA_ID: &str = "00012345600012";
const TEST_SCHEMA_NAME: &str = "Test Schema";
const TEST_SCHEMA_DESC : &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Ut et diam vehicula 
                                 felis. Cras sollicitudin, leo pretium viverra ultricies, orci dolor consequat massa, ac bibendum.";
                            
const TEST_SCHEMA_DESC_LONG : &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam eu rutrum libero.
                                     Quisque congue metus dictum ligula cursus maximus. Integer ultricies, magna eget 
                                     hendrerit vulputate, ligula ipsum elementum mi, nec porta leo metus at mauris. 
                                     Nunc a ultricies felis, sed eleifend ipsum. Donec at faucibus mi, ac tempus felis. Nunc.";
                                
const TEST_SCHEMA_VERSION: &str = "TS-v1.0";

#[test]
fn create_product_without_props() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let id = TEST_SCHEMA_ID.as_bytes().to_owned();
        let owner = account_key(TEST_ORGANIZATION);
        let hash = H256::from_low_u64_be(1);
        let now = 42;
        Timestamp::set_timestamp(now);

        let result = SchemaRegistry::register_schema(
            Origin::signed(sender),
            id.clone(),
            owner.clone(),
            hash.clone(),
            None,
        );

        assert_ok!(result);

        assert_eq!(
            SchemaRegistry::schema_by_id(&id),
            Some(Schema {
                id: id.clone(),
                owner: owner,
                hash: hash,
                registered: now,
                props: None
            })
        );

        assert_eq!(<SchemasOfOrganization<Test>>::get(owner), vec![id.clone()]);
        assert_eq!(SchemaRegistry::owner_of_schema(&id), Some(owner));
        assert_eq!(<SchemaByHash<Test>>::get(hash), vec![id.clone()]);

        // Event is raised
        assert!(System::events().iter().any(|er| er.event
            == TestEvent::schema_registry(RawEvent::SchemaRegistered(
                sender,
                id.clone(),
                hash.clone(),
            ))));
    });
}

#[test]
fn create_schema_with_valid_props() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let id = TEST_SCHEMA_ID.as_bytes().to_owned();
        let owner = account_key(TEST_ORGANIZATION);
        let hash = H256::from_low_u64_be(1);
        let now = 42;
        Timestamp::set_timestamp(now);

        let result = SchemaRegistry::register_schema(
            Origin::signed(sender),
            id.clone(),
            owner.clone(),
            hash.clone(),
            Some(vec![
                SchemaProperty::new(
                    &TEST_SCHEMA_NAME.as_bytes().to_owned(), 
                    &TEST_SCHEMA_DESC.as_bytes().to_owned(), 
                    &TEST_SCHEMA_VERSION.as_bytes().to_owned()),
            ]),
        );

        assert_ok!(result);

        assert_eq!(
            SchemaRegistry::schema_by_id(&id),
            Some(Schema {
                id: id.clone(),
                owner: owner,
                hash: hash.clone(),
                registered: now,
                props: Some(vec![
                    SchemaProperty::new(
                    &TEST_SCHEMA_NAME.as_bytes().to_owned(), 
                    &TEST_SCHEMA_DESC.as_bytes().to_owned(), 
                    &TEST_SCHEMA_VERSION.as_bytes().to_owned()),
                ]),
            })
        );

        assert_eq!(<SchemasOfOrganization<Test>>::get(owner), vec![id.clone()]);
        assert_eq!(SchemaRegistry::owner_of_schema(&id), Some(owner));
        assert_eq!(<SchemaByHash<Test>>::get(hash), vec![id.clone()]);

        // Event is raised
        assert!(System::events().iter().any(|er| er.event
            == TestEvent::schema_registry(RawEvent::SchemaRegistered(
                sender,
                id.clone(),
                hash.clone(),
            ))));
    });
}

#[test]
fn create_schema_with_invalid_sender() {
    new_test_ext().execute_with(|| {
        let hash = H256::from_low_u64_be(1);
        assert_noop!(
            SchemaRegistry::register_schema(
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
fn create_schema_with_missing_id() {
    new_test_ext().execute_with(|| {
        let hash = H256::from_low_u64_be(1);
        assert_noop!(
            SchemaRegistry::register_schema(
                Origin::signed(account_key(TEST_SENDER)),
                vec!(),
                account_key(TEST_ORGANIZATION),
                hash.clone(),
                None
            ),
            Error::<Test>::SchemaIdMissing
        );
    });
}

#[test]
fn create_schema_with_long_id() {
    new_test_ext().execute_with(|| {
        let hash = H256::from_low_u64_be(1);
        assert_noop!(
            SchemaRegistry::register_schema(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_SCHEMA_DESC.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                hash.clone(),
                None
            ),
            Error::<Test>::SchemaIdTooLong
        );
    })
}

#[test]
fn create_schema_with_existing_id() {
    new_test_ext().execute_with(|| {
        let existing_schema = TEST_SCHEMA_ID.as_bytes().to_owned();
        let hash = H256::from_low_u64_be(1);
        let now = 42;

        store_test_schema::<Test>(
            existing_schema.clone(),
            account_key(TEST_ORGANIZATION),
            hash.clone(),
            now,
        );

        assert_noop!(
            SchemaRegistry::register_schema(
                Origin::signed(account_key(TEST_SENDER)),
                existing_schema,
                account_key(TEST_ORGANIZATION),
                hash.clone(),
                None
            ),
            Error::<Test>::SchemaIdExists
        );
    })
}

#[test]
fn create_schema_with_too_many_props() {
    new_test_ext().execute_with(|| {
        let hash = H256::from_low_u64_be(1);
        assert_noop!(
            SchemaRegistry::register_schema(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_SCHEMA_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                hash.clone(),
                Some(vec![
                    SchemaProperty::new(b"name1", b"desc1", b"ver1"),
                    SchemaProperty::new(b"name2", b"desc2", b"ver2"),
                ])
            ),
            Error::<Test>::SchemaTooManyProps
        );
    })
}

#[test]
fn create_schema_with_invalid_name() {
    new_test_ext().execute_with(|| {
        let hash = H256::from_low_u64_be(1);
        assert_noop!(
            SchemaRegistry::register_schema(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_SCHEMA_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                hash.clone(),
                Some(vec![
                    SchemaProperty::new(&TEST_SCHEMA_DESC.as_bytes().to_owned(), b"desc1", b"ver1"),
                ])
            ),
            Error::<Test>::SchemaInvalidName
        );
    })
}

#[test]
fn create_schema_with_invalid_desc() {
    new_test_ext().execute_with(|| {
        let hash = H256::from_low_u64_be(1);
        assert_noop!(
            SchemaRegistry::register_schema(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_SCHEMA_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                hash.clone(),
                Some(vec![
                    SchemaProperty::new(b"name1", &TEST_SCHEMA_DESC_LONG.as_bytes().to_owned(), b"ver1"),
                ])
            ),
            Error::<Test>::SchemaInvalidDescription

        );
    })
}

#[test]
fn create_schema_with_invalid_version() {
    new_test_ext().execute_with(|| {
        let hash = H256::from_low_u64_be(1);
        assert_noop!(
            SchemaRegistry::register_schema(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_SCHEMA_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                hash.clone(),
                Some(vec![
                    SchemaProperty::new(b"name1", b"desc1", &TEST_SCHEMA_DESC.as_bytes().to_owned()),
                ])
            ),
            Error::<Test>::SchemaInvalidVersion
        );
    })
}
