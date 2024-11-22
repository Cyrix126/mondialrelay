// use std::{fs::File, io::Write};

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use url::Url;
use xsd_parser::generator::validator::Validate;

use crate::{
    db::{
        model::Shipment,
        schema::shipments::{self},
    },
    error::AppError,
    request::{Address, ShipmentCreationRequest},
    response::ShipmentCreationResponse,
    AppState,
};
#[derive(Deserialize, Serialize, Debug)]
pub struct NewShipment {
    pub id_order: u32,
    // should be a variant
    pub delivery_mode: String,
    // relay, or Auto if no relay used
    pub delivery_location: Option<String>,
    pub delivery_instructions: Option<String>,
    // cm
    pub length: u32,
    pub width: u32,
    pub depth: u32,
    pub weight: u32,
    pub recipient_details: Address,
}

// create a shipment
#[axum::debug_handler]
pub async fn shipment(
    State(state): State<AppState>,
    Json(data): Json<NewShipment>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Serving request for new shipment...");
    // validate NewShipment data,
    data.recipient_details
        .validate()
        .map_err(AppError::BadAddress)?;
    // save order id
    let order_id = data.id_order;
    // construct the request
    let shipment = ShipmentCreationRequest::new(state.config.clone(), data)?;
    // validate shipment request, return simple error to client, debugged error to server
    shipment.validate().map_err(AppError::Xml)?;

    // convert to xml
    let xml = yaserde::ser::to_string_with_config(
        &shipment,
        &yaserde::ser::Config {
            perform_indent: true,
            ..Default::default()
        },
    )
    .expect("invalid UTF-8");
    // send request
    let url = if state.config.test {
        "https://connect-api-sandbox.mondialrelay.com/api/shipment"
    } else {
        "https://connect-api.mondialrelay.com/api/shipment"
    };
    // debug response from test
    // let mut buffer = File::create("request_generated.xml").unwrap();
    // buffer.write_all(&xml.clone().into_bytes()).unwrap();
    let resp_xml = state
        .client
        .post(url)
        .body(xml)
        .send()
        .await?
        .text()
        .await?;
    debug!("response received:\n{}", &resp_xml);
    let resp =
        yaserde::de::from_str::<ShipmentCreationResponse>(&resp_xml).map_err(AppError::Xml)?;
    // check errors/warning in xml
    let label_url = resp
        .shipments_list
        .shipment
        .first()
        .expect("every request should have one shipment")
        .label_list
        .label
        .output
        .clone();

    // save id of order and label url in to db
    // tracking id is included in url of label
    let conn = state.pool.get().await?;
    let shipment = Shipment {
        order_id: order_id as i32,
        label_url: label_url.clone(),
        ..Default::default()
    };
    // wait the writing to finish, so client is sure the shipment is saved.
    conn.interact(move |conn| {
        diesel::insert_into(shipments::table)
            .values(shipment)
            .execute(conn)
    })
    .await??;
    let tracking = Url::parse(&label_url)
        .expect("label url given from mondialrelay should have a correct syntax")
        .query_pairs()
        .find(|(c, _)| c == "expedition")
        .expect("there should be always a expedition query")
        .1
        .to_string();

    debug!("Returning tracking id.");
    Ok(tracking)
}
/// returns label url for an order.
/// There can be multiple label for an order if multiple shipments has been created for one order.
#[axum::debug_handler]
pub async fn label(
    State(state): State<AppState>,
    Path(id_order): Path<u32>,
) -> Result<impl IntoResponse, AppError> {
    use crate::db::schema::shipments::dsl::*; // get url from order_id in db

    debug!("handling request \"Label\" for order n°{}", id_order);
    let conn = state.pool.get().await?;
    let labels = conn
        .interact(move |conn| {
            Ok::<Vec<String>, AppError>(
                shipments
                    .filter(order_id.eq(id_order as i32))
                    .select(label_url)
                    .load(conn)?,
            )
        })
        .await??;
    // return url
    if labels.is_empty() {
        warn!(
            "order n°{} label was requested but order does not exist !",
            id_order
        );
        return Err(AppError::OrderNotFound);
    }
    debug!("Returning label(s) for order n°{}", id_order);
    Ok(Json(labels))
}
