#[cfg(feature="protocol_v0")]
pub mod v0;


// this is a huge comppile time mapping table for underlying structures and types.
// it is not intended to be used directly by users of the library, but rather to provide a stable interface for the underlying implementation to interact with the public API. it is also intended to be used as a way to abstract away the underlying implementation details, so that the public API can remain stable even if the underlying implementation changes significantly. this is especially important for a library like this, 
// which is intended to be used in a wide variety of contexts and environments, and which may need to support multiple versions of the underlying implementation over time.

#[cfg(feature="protocol_v0")]
pub use v0::sqlite_zig_v0_init as sqlite_zig_init;
#[cfg(feature="protocol_v0")]
pub use v0::sqlite_zig_v0_shutdown as sqlite_zig_shutdown;
#[cfg(feature="protocol_v0")]
pub use v0::sqlite_zig_v0_configure as sqlite_zig_configure;
#[cfg(feature="protocol_v0")]
pub use v0::SqliteConfigResult as SqliteConfigResult;
#[cfg(feature="protocol_v0")]
pub use v0::SQliteZigConfigParameters as SQliteZigConfigParameters;
#[cfg(feature="protocol_v0")]
pub use v0::SQliteZigInitResult as SQliteZigInitResult;
#[cfg(feature="protocol_v0")]
pub use v0::SQliteZigShutdownResult as SQliteZigShutdownResult;
