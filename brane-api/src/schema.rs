table! {
    packages (id) {
        id -> Integer,
        created -> Timestamp,
        kind -> Text,
        name -> Text,
        uploaded -> Timestamp,
        uuid -> Text,
        version -> Text,
        description -> Nullable<Text>,
        functions_json -> Nullable<Text>,
        types_json -> Nullable<Text>,
        checksum -> BigInt,
        filename -> Text,
    }
}
