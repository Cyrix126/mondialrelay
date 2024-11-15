// test to check that a request with valid data will produce a valid response on Mondial Relay API.

use std::path::PathBuf;

use axum_test::TestServer;
use deadpool_diesel::postgres::Pool;
use diesel::RunQueryDsl;
use get_pass::get_password;
use mondialrelay_api_lib::{
    config::{Address, Config},
    db::schema::shipments,
    handler::NewShipment,
    request::{
        address_type::{
            CityType, CountryCodeType, FirstnameType, HouseNoType, LastnameType, PostCodeType,
            TitleType,
        },
        AddressType,
    },
    router, AppState,
};

#[tokio::test]
// requirements: having a postgresql db, create db mondialrelay and dev user with password available in pass at mondial/db/test. Having the .env file in the api crate with the DATABASE_URL var set.
async fn correct_response() -> Result<(), Box<dyn std::error::Error>> {
    let request = NewShipment {
        id_order: 1,
        delivery_mode: "24R".into(),
        delivery_location: Some("FR-24738".into()),
        delivery_instructions: None,
        length: 15,
        width: 10,
        depth: 5,
        weight: 150,
        recipient_details: AddressType {
            title: Some(TitleType("Mr".into())),
            firstname: Some(FirstnameType("John".into())),
            lastname: Some(LastnameType("LastName".into())),
            streetname: "RUE JEAN JACQUES ROUSSEAU".into(),
            house_no: Some(HouseNoType("84".into())),
            country_code: CountryCodeType("FR".into()),
            post_code: PostCodeType("21000".into()),
            city: CityType("Dijon".into()),
            ..Default::default()
        },
    };
    // load env file.
    let db_uri = dotenv::var("DATABASE_URL")
        .expect("Should have an .env file for the test database url.")
        .parse()
        .unwrap();
    let config = Config {
        db_uri,
        db_pass_path: "mondialrelay/db/test".into(),
        // not needed
        password_path_test: PathBuf::from("mondialrelay/db/test"),
        test: true,
        address_sender: Address {
            name_business: "Dupond".to_string(),
            streetname: "Rue du Berceau".into(),
            house_nb: 5,
            country_code: "FR".into(),
            post_code: "00000".into(),
            city: "Cityname".into(),
            phone_no: "+3980000000".into(),
            email: "test@example.com".into(),
        },
        ..Default::default()
    };
    // delete the tables at the beginning, in case it wasn't cleaned.
    delete_tables(&config).await;
    let state = AppState::new(config.clone()).await?;
    let app = TestServer::new(router(state))?;
    app.post("/shipment").json(&request).expect_success().await;
    delete_tables(&config).await;
    Ok(())
}
// mock server with config
// create the data for the request

// make the request

// check status of response
// reset data
async fn delete_tables(config: &Config) {
    let mut db_uri = config.db_uri.clone();
    db_uri
        .set_password(Some(
            &get_password(&config.db_pass_path).expect("Invalid utf-8"),
        ))
        .unwrap();
    dbg!(&db_uri);
    let pool = Pool::builder(deadpool_diesel::Manager::new(
        db_uri.as_str(),
        deadpool_diesel::Runtime::Tokio1,
    ))
    .build()
    .unwrap();
    let conn = pool.get().await.unwrap();
    conn.interact(move |conn| diesel::delete(shipments::table).execute(conn))
        .await
        .unwrap()
        .unwrap();
}
