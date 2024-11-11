use axum::{extract::State, response::IntoResponse, Json};
use diesel::RunQueryDsl;
use serde::Deserialize;
use url::Url;
use xsd_parser::generator::validator::Validate;

use crate::{
    db::{model::Shipment, schema::shipments},
    error::AppError,
    request::{
        output_options_type::{OutputFormatType, OutputTypeType},
        shipment_type::{DeliveryInstructionType, ParcelCountType},
        AddressType, MeasureAmountType, OutputOptionsType, ParcelListType, ParcelType,
        ProductConfigurationType, SenderDetailsType, ShipmentCreationRequest, ShipmentType,
        ShipmentsListType,
    },
    response::ShipmentCreationResponseType,
    AppState,
};
#[derive(Deserialize)]
pub struct NewShipment {
    id_order: u32,
    // should be a variant
    delivery_mode: String,
    // relay, or Auto if no relay used
    delivery_location: Option<String>,
    delivery_instructions: Option<String>,
    // cm
    length: u32,
    width: u32,
    depth: u32,
    weight: u32,
    recipient_details: AddressType,
}

// create a shipment
#[axum::debug_handler]
pub async fn shipment(
    State(state): State<AppState>,
    Json(data): Json<NewShipment>,
) -> Result<impl IntoResponse, AppError> {
    // construct the request
    let shipment = ShipmentCreationRequest {
        context: state
            .config
            .context_api_mondialrelay()
            .map_err(|_| AppError::Conf)?,
        output_options: OutputOptionsType {
            output_format: OutputFormatType(state.config.format.clone()),
            output_type: OutputTypeType("PdfUrl".to_string()),
        },
        shipments_list: ShipmentsListType {
            shipment: vec![ShipmentType {
                // MondialRelay doesn't need to know our customer id nor order id
                order_no: None,
                customer_no: None,
                parcel_count: ParcelCountType(1),
                shipment_value: None,
                options: None,
                delivery_mode: ProductConfigurationType {
                    mode: data.delivery_mode,
                    location: data.delivery_location,
                },
                collection_mode: ProductConfigurationType {
                    mode: "CCC".to_string(),
                    location: None,
                },
                parcels: ParcelListType {
                    parcel: vec![ParcelType {
                        content: None,
                        length: MeasureAmountType {
                            value: data.length as f64,
                            unit: "cm".to_string(),
                        },
                        width: MeasureAmountType {
                            value: data.width as f64,
                            unit: "cm".to_string(),
                        },
                        depth: MeasureAmountType {
                            value: data.depth as f64,
                            unit: "cm".to_string(),
                        },
                        weight: MeasureAmountType {
                            value: data.weight as f64,
                            unit: "cm".to_string(),
                        },
                    }],
                },
                delivery_instruction: data.delivery_instructions.map(DeliveryInstructionType),
                sender: SenderDetailsType {
                    address: state.config.sender_address(),
                },
                recipient: crate::request::RecipientDetailsType {
                    address: data.recipient_details,
                },
            }],
        },
    };
    // validate
    shipment.validate().map_err(AppError::Xml)?;

    // convert to xml
    let xml = yaserde::ser::to_string(&shipment).expect("invalid UTF-8");
    // send request
    let url = if state.config.test {
        "https://connect-api-sandbox.mondialrelay.com/api/shipment"
    } else {
        "https://connect-api.mondialrelay.com/api/shipment"
    };
    let resp_xml = state.client.get(url).body(xml).send().await?.text().await?;
    dbg!(&resp_xml);
    let resp =
        yaserde::de::from_str::<ShipmentCreationResponseType>(&resp_xml).map_err(AppError::Xml)?;
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
        order_id: data.id_order as i32,
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

    Ok(tracking)
}
