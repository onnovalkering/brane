table! {
    invocations (id) {
        id -> Int4,
        session -> Int4,
        created -> Timestamp,
        name -> Nullable<Varchar>,
        started -> Nullable<Timestamp>,
        stopped -> Nullable<Timestamp>,
        uuid -> Varchar,
        instructions_json -> Text,
        status -> Varchar,
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
        source -> Nullable<Text>,
        types_json -> Nullable<Text>,
        checksum -> Int8,
        filename -> Varchar,
    }
}

table! {
    sessions (id) {
        id -> Int4,
        created -> Timestamp,
        uuid -> Varchar,
        status -> Varchar,
    }
}

table! {
    variables (id) {
        id -> Int4,
        session -> Int4,
        created -> Timestamp,
        updated -> Nullable<Timestamp>,
        name -> Varchar,
        #[sql_name = "type"]
        type_ -> Varchar,
        content_json -> Nullable<Text>,
    }
}

joinable!(invocations -> sessions (session));
joinable!(variables -> sessions (session));

allow_tables_to_appear_in_same_query!(
    invocations,
    packages,
    sessions,
    variables,
);
