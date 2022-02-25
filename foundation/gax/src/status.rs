use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// A gRPC status describing the result of an RPC call.
pub struct Status {
    /// Optional underlying error.
    pub source: tonic::Status,
}

impl Status {
    pub fn new(cause: tonic::Status) -> Self {
        Status { source: cause }
    }
    /// Get the gRPC `Code` of this `Status`.
    pub fn code(&self) -> Code {
        self.source.code().into()
    }

    /// Get the text error message of this `Status`.
    pub fn message(&self) -> &str {
        &self.source.message()
    }

    /// Get the opaque error details of this `Status`.
    pub fn details(&self) -> &[u8] {
        &self.source.details()
    }
}

impl Error for Status {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.source()
    }
}

impl From<tonic::Status> for Status {
    fn from(tonic_status: tonic::Status) -> Self {
        return Status { source: tonic_status };
    }
}

impl Debug for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.source, f)
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.source, f)
    }
}

/// gRPC status codes used by [`Status`].
///
/// These variants match the [gRPC status codes].
///
/// [gRPC status codes]: https://github.com/grpc/grpc/blob/master/doc/statuscodes.md#status-codes-and-their-use-in-grpc
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Code {
    /// The operation completed successfully.
    Ok = 0,

    /// The operation was cancelled.
    Cancelled = 1,

    /// Unknown error.
    Unknown = 2,

    /// Client specified an invalid argument.
    InvalidArgument = 3,

    /// Deadline expired before operation could complete.
    DeadlineExceeded = 4,

    /// Some requested entity was not found.
    NotFound = 5,

    /// Some entity that we attempted to create already exists.
    AlreadyExists = 6,

    /// The caller does not have permission to execute the specified operation.
    PermissionDenied = 7,

    /// Some resource has been exhausted.
    ResourceExhausted = 8,

    /// The system is not in a state required for the operation's execution.
    FailedPrecondition = 9,

    /// The operation was aborted.
    Aborted = 10,

    /// Operation was attempted past the valid range.
    OutOfRange = 11,

    /// Operation is not implemented or not supported.
    Unimplemented = 12,

    /// Internal error.
    Internal = 13,

    /// The service is currently unavailable.
    Unavailable = 14,

    /// Unrecoverable data loss or corruption.
    DataLoss = 15,

    /// The request does not have valid authentication credentials
    Unauthenticated = 16,
}

impl From<tonic::Code> for Code {
    fn from(tonic_code: tonic::Code) -> Self {
        match tonic_code {
            tonic::Code::Ok => Code::Ok,
            tonic::Code::Cancelled => Code::Cancelled,
            tonic::Code::Unknown => Code::Unknown,
            tonic::Code::InvalidArgument => Code::InvalidArgument,
            tonic::Code::DeadlineExceeded => Code::DeadlineExceeded,
            tonic::Code::NotFound => Code::NotFound,
            tonic::Code::AlreadyExists => Code::AlreadyExists,
            tonic::Code::PermissionDenied => Code::PermissionDenied,
            tonic::Code::ResourceExhausted => Code::ResourceExhausted,
            tonic::Code::FailedPrecondition => Code::FailedPrecondition,
            tonic::Code::Aborted => Code::Aborted,
            tonic::Code::OutOfRange => Code::OutOfRange,
            tonic::Code::Unimplemented => Code::Unimplemented,
            tonic::Code::Internal => Code::Internal,
            tonic::Code::Unavailable => Code::Unavailable,
            tonic::Code::DataLoss => Code::DataLoss,
            tonic::Code::Unauthenticated => Code::Unauthenticated,
        }
    }
}

impl Code {
    pub fn description(&self) -> &'static str {
        match self {
            Code::Ok => "The operation completed successfully",
            Code::Cancelled => "The operation was cancelled",
            Code::Unknown => "Unknown error",
            Code::InvalidArgument => "Client specified an invalid argument",
            Code::DeadlineExceeded => "Deadline expired before operation could complete",
            Code::NotFound => "Some requested entity was not found",
            Code::AlreadyExists => "Some entity that we attempted to create already exists",
            Code::PermissionDenied => "The caller does not have permission to execute the specified operation",
            Code::ResourceExhausted => "Some resource has been exhausted",
            Code::FailedPrecondition => "The system is not in a state required for the operation's execution",
            Code::Aborted => "The operation was aborted",
            Code::OutOfRange => "Operation was attempted past the valid range",
            Code::Unimplemented => "Operation is not implemented or not supported",
            Code::Internal => "Internal error",
            Code::Unavailable => "The service is currently unavailable",
            Code::DataLoss => "Unrecoverable data loss or corruption",
            Code::Unauthenticated => "The request does not have valid authentication credentials",
        }
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.description(), f)
    }
}
