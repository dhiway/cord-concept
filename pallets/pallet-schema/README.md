# Cord Schema Registry pallet

The Schema Registry pallet provides functionality for registering and managing master schemas describing credentials used across various domains. This data is typically registered once by the chain admin to be shared with other network participants.

NOTE: This pallet implements the logic in a simplified way, thus it is not audited or ready for production use.

## Usage

To register a schema, one must send a transaction with a `schemaRegistry.registerSchema` extrinsic with the following arguments:
- `id` as the Schema ID, typically this would be a numeric or alpha-numeric code with a well-defined data structure.
- `owner` as the Substrate Account representing the organization created this schema, 
- `hash` hash of the schema content
- `props` which is a series of properties (name,value & ver) describing the schema. 

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
[dependencies.pallet-schema]
default-features = false 
git = 'https://github.com/dhiway/pallet-schema.git'
package = 'pallet-schema'
version = '0.0.1'
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'pallet-schema/std',
]
```

### Runtime `lib.rs`

You should implement it's trait like so:

```rust
impl schema_registry::Trait for Runtime {
	type Event = Event;
	type CreateRoleOrigin = Origin;
}
```

and include it in your `construct_runtime!` macro:

```rust
SchemaRegistry: Schema_registry::{Module, Call, Storage, Event<T>},
```

### Genesis Configuration

This template pallet does not have any genesis configuration.

