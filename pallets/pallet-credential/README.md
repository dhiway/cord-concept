# Cord Credential Hash Registry pallet

This Hash Registry pallet provides functionality for writing the hash of the credential issued. This data is typically registered by the approved signing authority in a subscribed organisation.

NOTE: This pallet implements the logic in a simplified way, thus it is not audited or ready for production use.

## Usage

To register a credential, one must send a transaction with a `credentialRegistry.registerCredential` extrinsic with the following arguments:
- `id` as the Cred ID, typically this would be a numeric or alpha-numeric code with a well-defined data structure.
- `owner` as the Substrate Account representing the organization created this credential, 
- `hash` hash of the schema content
- `props` which is a series of properties (subject) describing the credential. 

## Dependencies

### Traits

This pallet depends on on the [FRAME EnsureOrigin System trait]
```
frame_support::traits::EnsureOrigin;
```

### Pallets

This pallet depends on on the [FRAME Timestamp pallet](https://docs.rs/crate/pallet-timestamp).

## Testing

Run the tests with:

    ```
    cargo test
    ```
## Building

Run the build with:

    ```
    cargo build --release
    ```
  
## How to use in your runtime

### Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following to your runtime's `Cargo.toml` file:

```TOML
[dependencies.pallet-credential]
default-features = false 
git = 'https://github.com/dhiway/pallet-schema.git'
package = 'pallet-credential'
version = '0.0.1'
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'pallet-credential/std',
]
```

### Runtime `lib.rs`

You should implement it's trait like so:

```rust
impl credential_registry::Trait for Runtime {
	type Event = Event;
	type CreateRoleOrigin = Origin;
}
```

and include it in your `construct_runtime!` macro:

```rust
CredentialRegistry: Credential_registry::{Module, Call, Storage, Event<T>},
```

### Genesis Configuration

This template pallet does not have any genesis configuration.

