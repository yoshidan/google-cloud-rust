/// Represents a textual expression in the Common Expression Language (CEL)
/// syntax. CEL is a C-like expression language. The syntax and semantics of CEL
/// are documented at <https://github.com/google/cel-spec.>
///
/// Example (Comparison):
///
///      title: "Summary size limit"
///      description: "Determines if a summary is less than 100 chars"
///      expression: "document.summary.size() < 100"
///
/// Example (Equality):
///
///      title: "Requestor is owner"
///      description: "Determines if requestor is the document owner"
///      expression: "document.owner == request.auth.claims.email"
///
/// Example (Logic):
///
///      title: "Public documents"
///      description: "Determine whether the document should be publicly visible"
///      expression: "document.type != 'private' && document.type != 'internal'"
///
/// Example (Data Manipulation):
///
///      title: "Notification string"
///      description: "Create a notification string with a timestamp."
///      expression: "'New message received at ' + string(document.create_time)"
///
/// The exact variables and functions that may be referenced within an expression
/// are determined by the service that evaluates it. See the service
/// documentation for additional information.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Expr {
    /// Textual representation of an expression in Common Expression Language
    /// syntax.
    #[prost(string, tag = "1")]
    pub expression: ::prost::alloc::string::String,
    /// Optional. Title for the expression, i.e. a short string describing
    /// its purpose. This can be used e.g. in UIs which allow to enter the
    /// expression.
    #[prost(string, tag = "2")]
    pub title: ::prost::alloc::string::String,
    /// Optional. Description of the expression. This is a longer text which
    /// describes the expression, e.g. when hovered over it in a UI.
    #[prost(string, tag = "3")]
    pub description: ::prost::alloc::string::String,
    /// Optional. String indicating the location of the expression for error
    /// reporting, e.g. a file name and a position in the file.
    #[prost(string, tag = "4")]
    pub location: ::prost::alloc::string::String,
}
/// Represents a whole or partial calendar date, such as a birthday. The time of
/// day and time zone are either specified elsewhere or are insignificant. The
/// date is relative to the Gregorian Calendar. This can represent one of the
/// following:
///
/// * A full date, with non-zero year, month, and day values
/// * A month and day value, with a zero year, such as an anniversary
/// * A year on its own, with zero month and day values
/// * A year and month value, with a zero day, such as a credit card expiration
/// date
///
/// Related types are \[google.type.TimeOfDay][google.type.TimeOfDay\] and
/// `google.protobuf.Timestamp`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Date {
    /// Year of the date. Must be from 1 to 9999, or 0 to specify a date without
    /// a year.
    #[prost(int32, tag = "1")]
    pub year: i32,
    /// Month of a year. Must be from 1 to 12, or 0 to specify a year without a
    /// month and day.
    #[prost(int32, tag = "2")]
    pub month: i32,
    /// Day of a month. Must be from 1 to 31 and valid for the year and month, or 0
    /// to specify a year by itself or a year and month where the day isn't
    /// significant.
    #[prost(int32, tag = "3")]
    pub day: i32,
}
