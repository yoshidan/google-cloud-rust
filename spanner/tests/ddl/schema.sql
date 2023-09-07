CREATE TABLE User
(
    UserId STRING(36) NOT NULL,
    NotNullINT64 INT64 NOT NULL,
    NullableINT64 INT64,
    NotNullFloat64 FLOAT64 NOT NULL,
    NullableFloat64 FLOAT64,
    NotNullBool BOOL NOT NULL,
    NullableBool BOOL,
    NotNullByteArray BYTES(MAX) NOT NULL,
    NullableByteArray BYTES(MAX),
    NotNullNumeric NUMERIC NOT NULL,
    NullableNumeric NUMERIC,
    NotNullTimestamp TIMESTAMP NOT NULL,
    NullableTimestamp TIMESTAMP,
    NotNullDate DATE NOT NULL,
    NullableDate DATE,
    NotNullArray Array<INT64> NOT NULL,
    NullableArray Array<INT64>,
    NullableString STRING(100),
    UpdatedAt TIMESTAMP NOT NULL OPTIONS (allow_commit_timestamp=true)
) PRIMARY KEY(UserId);

CREATE TABLE UserItem
(
    UserId STRING(36) NOT NULL,
    ItemId INT64 NOT NULL,
    Quantity INT64 NOT NULL,
    UpdatedAt TIMESTAMP NOT NULL OPTIONS (allow_commit_timestamp=true)
) PRIMARY KEY(UserId, ItemId), INTERLEAVE IN PARENT User ON DELETE CASCADE;

CREATE TABLE UserItemHistory
(
    UserId STRING(36) NOT NULL,
    ItemId INT64 NOT NULL,
    UsedAt TIMESTAMP NOT NULL OPTIONS (allow_commit_timestamp=true)
) PRIMARY KEY(UserId, ItemId, UsedAt), INTERLEAVE IN PARENT UserItem ON DELETE CASCADE;

CREATE TABLE UserCharacter
(
    UserId STRING(36) NOT NULL,
    CharacterId INT64 NOT NULL,
    Level INT64 NOT NULL,
    UpdatedAt TIMESTAMP NOT NULL OPTIONS (allow_commit_timestamp=true)
) PRIMARY KEY(UserId, CharacterId), INTERLEAVE IN PARENT User ON DELETE CASCADE;

CREATE TABLE Guild
(
    GuildId STRING(36) NOT NULL,
    OwnerUserId STRING(36) NOT NULL,
    UpdatedAt TIMESTAMP NOT NULL OPTIONS (allow_commit_timestamp=true)
) PRIMARY KEY(GuildId);