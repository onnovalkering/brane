table! {
    packages (id) {
        id -> Integer,
        uuid -> Text,
        kind -> Text,
        name -> Text,
        version -> Text,
        created -> Timestamp,
        description -> Nullable<Text>,
        functions_json -> Nullable<Text>,
        types_json -> Nullable<Text>,
    }
}
