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
