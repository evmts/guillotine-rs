//! Error types for the guillotine-mini REVM adapter

#[derive(Debug)]
pub enum EvmAdapterError<DbErr> {
    /// Database-related error from REVM
    Db(DbErr),
    /// FFI call failed (bool=false or null handle)
    Ffi(&'static str),
}

impl<DbErr: core::fmt::Debug> core::fmt::Display for EvmAdapterError<DbErr> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Db(e) => write!(f, "database error: {:?}", e),
            Self::Ffi(name) => write!(f, "ffi call failed: {}", name),
        }
    }
}

impl<DbErr: core::fmt::Debug> std::error::Error for EvmAdapterError<DbErr> {}

