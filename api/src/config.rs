use serde::{Deserialize, Serialize};
use std::{error::Error, path::PathBuf};
use url::Url;

use crate::request::{
    address_type::CountryCodeType,
    context_type::{CultureType, CustomerIdType, VersionAPIType},
    AddressType, ContextType,
};

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    // cover database connection
    pub db_uri: Url,
    pub db_pass_path: PathBuf,
    // port on which the cover API will listen for incoming connections
    pub listen_port: u16,
    // logins for mondialrelay
    pub brand_id: String,
    pub password_path: PathBuf,
    pub password_path_test: PathBuf,
    // Mondial Relay language of printed label. en-EN format.
    pub culture: String,
    // Mondial Relay label output: A4, A5, 10x15
    pub format: String,
    // sender details
    pub address_sender: Address,
    // are we in test mode ?
    pub test: bool,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Address {
    pub name_business: String,
    pub streetname: String,
    pub house_nb: u32,
    // The two letter country code of the addressee (e. g. DE, GB). For a
    // complete list of country code, refer to the standard ISO 3166-1-alpha-2
    pub country_code: String,
    pub post_code: String,
    pub city: String,
    // The phone number of the addressee. Please
    // specify the area code (e.g. +33 for FRANCE).
    pub phone_no: String,
    pub email: String,
}

impl Default for Config {
    // default values will use test instance for mondialrelay
    fn default() -> Self {
        Self {
            db_uri: Url::parse("postgresql://user@127.0.0.1:5432/mydb").unwrap(),
            db_pass_path: PathBuf::from("name_api/db/user"),
            listen_port: 10200,
            brand_id: String::from("BDTEST"),
            password_path_test: PathBuf::from("mondialrelay_api_test"),
            password_path: PathBuf::from("mondialrelay_api"),
            culture: String::from("fr-FR"),
            format: "A4".to_string(),
            // todo example address
            address_sender: Address::default(),
            test: true,
        }
    }
}

impl Config {
    pub fn context_api_mondialrelay(&self) -> Result<ContextType, Box<dyn Error>> {
        let brand_id = if self.test { "BDTEST" } else { &self.brand_id };
        let login = [brand_id, "@business-api.mondialrelay.com"].concat();
        let pass_path = if self.test {
            &self.password_path_test
        } else {
            &self.password_path
        };
        Ok(ContextType {
            login,
            password: get_pass::get_password(pass_path)?,
            customer_id: CustomerIdType(self.brand_id.clone()),
            culture: CultureType(self.culture.clone()),
            version_api: VersionAPIType("1.0".to_string()),
        })
    }
    pub fn sender_address(&self) -> AddressType {
        let adr = self.address_sender.clone();
        AddressType {
            title: None,
            firstname: None,
            lastname: None,
            streetname: adr.streetname,
            house_no: Some(crate::request::address_type::HouseNoType(
                adr.house_nb.to_string(),
            )),
            country_code: CountryCodeType(adr.country_code),
            post_code: crate::request::address_type::PostCodeType(adr.post_code),
            city: crate::request::address_type::CityType(adr.city),
            address_add_1: Some(crate::request::address_type::AddressAdd1Type(
                adr.name_business,
            )),
            address_add_2: None,
            address_add_3: None,
            phone_no: crate::request::address_type::PhoneNoType(adr.phone_no),
            mobile_no: None,
            email: Some(crate::request::address_type::EmailType(adr.email)),
        }
    }
}
