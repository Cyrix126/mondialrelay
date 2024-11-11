// @generated automatically by Diesel CLI.

diesel::table! {
    shipments (id) {
        id -> Int4,
        order_id -> Int4,
        label_url -> Text,
        created_at -> Timestamptz,
    }
}
