table! {
    invocations (id) {
        id -> Int4,
        created -> Timestamp,
        name -> Nullable<Varchar>,
        uuid -> Varchar,
        status -> Varchar,
        arguments_json -> Text,
        instructions_json -> Text,
    }
}

table! {
    packages (id) {
        id -> Int4,
        created -> Timestamp,
        kind -> Varchar,
        name -> Varchar,
        uploaded -> Timestamp,
        uuid -> Varchar,
        version -> Varchar,
        description -> Nullable<Varchar>,
        functions_json -> Nullable<Text>,
        types_json -> Nullable<Text>,
        checksum -> Int8,
        filename -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(invocations, packages,);
