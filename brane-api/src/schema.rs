table! {
    invocations (id) {
        id -> Integer,
        created -> Timestamp,
        name -> Nullable<Text>,
        uuid -> Text,
        status -> Text,
        arguments_json -> Text,
        instructions_json -> Text,
    }
}

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

allow_tables_to_appear_in_same_query!(
    invocations,
    packages,
);
