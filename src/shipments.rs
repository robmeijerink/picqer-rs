//! Shipments.
//!
//! A shipment asks a shipping provider for a label for a picklist. Picqer describes the fields in the
//! documentation, and the example responses show more, which are read here too. The fields the
//! documentation does not show at all, such as the URL of the label in a shipment itself, are not
//! read: the documents of a parcel say where the label is.
//!
//! The shipments in this module were checked with the `Manual` shipping provider of a developer
//! account. The `PostNL` provider of that account has no valid key, so a shipment with it fails.

use std::collections::BTreeMap;

use reqwest::Method;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::client::Client;
use crate::error::Result;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::reported::Reported;
use crate::types::{
    DateTime, FulfilmentCustomerId, Grams, NonEmpty, OrderId, PackagingId, ParcelDocumentId,
    ParcelId, PicklistId, ReturnId, ShipmentId, ShippingProviderId, ShippingProviderProfileId,
    UserId,
};

string_enum! {
    /// The carrier that delivers a shipment. Picqer uses it to show the logo of the carrier and to
    /// tell shops who the carrier is.
    pub enum CarrierKey {
        /// Ampere.
        Ampere = "ampere",
        /// bpost.
        Bpost = "bpost",
        /// Budbee.
        Budbee = "budbee",
        /// DHL Germany.
        DhlDe = "dhl_de",
        /// DHL For You.
        DhlForYou = "dhl_for_you",
        /// DHL Netherlands.
        DhlNl = "dhl_nl",
        /// DPD Belgium.
        DpdBe = "dpd_be",
        /// DPD Netherlands.
        DpdNl = "dpd_nl",
        /// GLS.
        Gls = "gls",
        /// Packs.
        Packs = "packs",
        /// PostNL.
        Postnl = "postnl",
        /// Maersk.
        Maersk = "maersk",
        /// Red je Pakketje.
        RedJePakketje = "red_je_pakketje",
        /// Sandd2me.
        Sandd2me = "sandd2me",
        /// Transmission.
        Transmission = "transmission",
        /// Trunkrs.
        Trunkrs = "trunkrs",
        /// UPS.
        Ups = "ups",
        /// A postal service.
        Postal = "postal",
        /// A courier.
        Courier = "courier",
        /// Another carrier.
        Other = "other",
    }
}

string_enum! {
    /// What a document of a parcel is.
    pub enum DocumentType {
        /// A shipping label.
        Label = "label",
        /// A customs document.
        Customs = "customs",
        /// Another document.
        Other = "other",
    }
}

string_enum! {
    /// The paper size of a document of a parcel.
    pub enum DocumentSize {
        /// A6.
        A6 = "A6",
        /// A4.
        A4 = "A4",
    }
}

string_enum! {
    /// The format of a document of a parcel.
    pub enum DocumentFormat {
        /// A PDF.
        Pdf = "pdf",
        /// A ZPL file for a label printer.
        Zpl = "zpl",
    }
}

/// A document that belongs to a parcel, such as its shipping label.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ParcelDocument {
    /// The document.
    pub idparcel_document: ParcelDocumentId,
    /// What the document is.
    #[serde(rename = "type")]
    pub document_type: Option<Reported<DocumentType>>,
    /// The paper size.
    pub size: Option<Reported<DocumentSize>>,
    /// The format.
    pub format: Option<Reported<DocumentFormat>>,
    /// Where the document can be downloaded.
    pub url: Option<Url>,
}

/// A parcel of a shipment.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Parcel {
    /// The parcel.
    pub idparcel: ParcelId,
    /// The packaging of the parcel.
    pub idpackaging: Option<PackagingId>,
    /// The weight in grams.
    pub weight: Option<Grams>,
    /// The tracking number from the shipping provider.
    pub tracking_code: Option<String>,
    /// Where the parcel can be tracked.
    pub tracking_url: Option<String>,
    /// The documents of the parcel.
    #[serde(default)]
    pub documents: Vec<ParcelDocument>,
}

/// A shipment of a picklist.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Shipment {
    /// The shipment.
    pub idshipment: ShipmentId,
    /// The picklist the shipment is for.
    pub idpicklist: Option<PicklistId>,
    /// The order the shipment is for.
    pub idorder: Option<OrderId>,
    /// The return the shipment is for.
    pub idreturn: Option<ReturnId>,
    /// The packaging of the shipment.
    pub idpackaging: Option<PackagingId>,
    /// The shipping provider that made the shipment.
    pub idcompany_shippingprovider: Option<ShippingProviderId>,
    /// The profile of the shipping provider that was used.
    pub idcompany_shippingprovider_profile: Option<ShippingProviderProfileId>,
    /// The kind of shipping provider, such as `PostNL` or `Manual`.
    pub provider: Option<String>,
    /// The name given to the provider.
    pub providername: Option<String>,
    /// The name of the service or provider as the public knows it.
    pub public_providername: Option<String>,
    /// The name of the profile that was used.
    pub profile_name: Option<String>,
    /// The carrier that delivers the shipment.
    pub carrier_key: Option<Reported<CarrierKey>>,
    /// The weight in grams that was given when the shipment was made and passed on to the shipping
    /// provider. Picqer leaves it out for shipments from before 4 December 2023.
    pub weight: Option<Grams>,
    /// The user that made the shipment.
    pub created_by_iduser: Option<UserId>,
    /// Whether the shipment is cancelled. Picqer says this can be done within five minutes after
    /// making the shipment.
    pub cancelled: bool,
    /// When the shipment was made.
    pub created: Option<DateTime>,
    /// When the shipment was last changed.
    pub updated: Option<DateTime>,
    /// The fulfilment customer the shipment belongs to, with Picqer Fulfilment.
    pub idfulfilment_customer: Option<FulfilmentCustomerId>,
    /// The parcels of the shipment.
    #[serde(default)]
    pub parcels: Vec<Parcel>,
}

/// A field a shipping method asks for when a shipment is made.
///
/// The documentation only shows an empty list of these. The fields here are what a developer
/// account answers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ExtraField {
    /// The key to use in [`NewShipment::extrafield`].
    pub key: String,
    /// The name of the field.
    pub name: Option<String>,
    /// A description of the field.
    pub description: Option<String>,
    /// Whether the shipping provider needs the field.
    pub is_required: Option<bool>,
    /// The value Picqer uses when none is given.
    #[serde(default, deserialize_with = "crate::serde_util::opt_text")]
    pub default_value: Option<String>,
}

/// A way of shipping that is allowed for a picklist.
///
/// The documentation shows the profile, the name, the provider, `selected` and `extra_fields`.
/// `multi_colli_supported` and the contents of `extra_fields` are not in it: they are what a
/// developer account answers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ShippingMethod {
    /// The profile to make a shipment with.
    pub idshippingprovider_profile: ShippingProviderProfileId,
    /// The name of the method.
    pub name: Option<String>,
    /// The provider.
    pub provider: Option<String>,
    /// Whether this is the method Picqer selected for the picklist.
    pub selected: Option<bool>,
    /// Whether the provider can ship a picklist in several parcels.
    pub multi_colli_supported: Option<bool>,
    /// The fields the method asks for.
    #[serde(default)]
    pub extra_fields: Vec<ExtraField>,
}

/// Filters for listing shipments.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShipmentQuery {
    search: Option<String>,
    since: Option<DateTime>,
    until: Option<DateTime>,
}

impl ShipmentQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Searches the tracking codes and the delivery names of the picklists the shipments are for.
    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    /// Lists the shipments created after this moment.
    pub fn since(mut self, moment: DateTime) -> Self {
        self.since = Some(moment);
        self
    }

    /// Lists the shipments created before this moment.
    pub fn until(mut self, moment: DateTime) -> Self {
        self.until = Some(moment);
        self
    }

    pub(crate) fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(search) = &self.search {
            pairs.push(("search", search.clone()));
        }
        if let Some(moment) = self.since {
            pairs.push(("sincedate", moment.to_string()));
        }
        if let Some(moment) = self.until {
            pairs.push(("untildate", moment.to_string()));
        }
        pairs
    }
}

/// A parcel of a shipment to make in several parcels.
///
/// This follows the example in the documentation, which gives every parcel a packaging and a weight.
/// The `Manual` provider of the developer account accepts one parcel and refuses two or more with
/// code 26, because it does not support several parcels, so a shipment with several parcels could
/// not be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NewParcel {
    idpackaging: PackagingId,
    weight: Grams,
}

impl NewParcel {
    /// A parcel in a packaging with a weight.
    pub fn new(idpackaging: PackagingId, weight: Grams) -> Self {
        Self {
            idpackaging,
            weight,
        }
    }
}

/// A shipment to make for a picklist.
///
/// Picqer sends the request to the shipping provider, which can take up to 20 seconds, and answers
/// with the error of the provider when it fails. A shipping profile is required. Picqer truncates a
/// fraction of the weight and takes a negative weight as zero, which cannot be sent here, and it
/// ignores an extra field the profile does not ask for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewShipment {
    idshippingprofile: ShippingProviderProfileId,
    #[serde(skip_serializing_if = "Option::is_none")]
    idpackaging: Option<PackagingId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    weight: Option<Grams>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    extrafields: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parcels: Option<NonEmpty<NewParcel>>,
}

impl NewShipment {
    /// A shipment with a shipping profile. See [`Shipments::methods`] for the profiles that are
    /// allowed for a picklist.
    pub fn new(idshippingprofile: ShippingProviderProfileId) -> Self {
        Self {
            idshippingprofile,
            idpackaging: None,
            weight: None,
            extrafields: BTreeMap::new(),
            parcels: None,
        }
    }

    /// Sets the packaging.
    pub fn idpackaging(mut self, idpackaging: PackagingId) -> Self {
        self.idpackaging = Some(idpackaging);
        self
    }

    /// Sets the weight.
    pub fn weight(mut self, weight: Grams) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Sets an extra field the shipping method asks for. The `Manual` provider takes `code`, the
    /// tracking code, and `tracking_url`.
    pub fn extrafield(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extrafields.insert(key.into(), value.into());
        self
    }

    /// Makes the shipment in several parcels, for a provider that supports that.
    pub fn parcels(mut self, parcels: NonEmpty<NewParcel>) -> Self {
        self.parcels = Some(parcels);
        self
    }
}

pub(crate) fn list(query: &ShipmentQuery, page: Page) -> Call<Vec<Shipment>> {
    let mut request = Request::new(Method::GET, segments(&[&"shipments"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: ShipmentId) -> Call<Shipment> {
    Call::new(Request::new(Method::GET, segments(&[&"shipments", &id])))
}

pub(crate) fn for_picklist(id: PicklistId) -> Call<Vec<Shipment>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"picklists", &id, &"shipments"]),
    ))
}

pub(crate) fn methods(id: PicklistId) -> Call<Vec<ShippingMethod>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"picklists", &id, &"shippingmethods"]),
    ))
}

pub(crate) fn create(id: PicklistId, shipment: &NewShipment) -> Result<Call<Shipment>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"picklists", &id, &"shipments"]))
            .query("return", "shipment")
            .json_body(shipment)?,
    ))
}

macro_rules! shipments_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of shipments, newest first.
        pub $($async)? fn list(
            &self,
            query: &$crate::shipments::ShipmentQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::shipments::Shipment>> {
            self.client.execute($crate::shipments::list(query, page)) $(.$await)?
        }

        /// Lists all shipments that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::shipments::ShipmentQuery,
        ) -> $crate::error::Result<Vec<$crate::shipments::Shipment>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::shipments::list(query, page)) $(.$await)?)
        }

        /// Fetches a shipment.
        pub $($async)? fn get(
            &self,
            id: $crate::types::ShipmentId,
        ) -> $crate::error::Result<$crate::shipments::Shipment> {
            self.client.execute($crate::shipments::get(id)) $(.$await)?
        }

        /// Lists the shipments of a picklist.
        pub $($async)? fn for_picklist(
            &self,
            picklist: $crate::types::PicklistId,
        ) -> $crate::error::Result<Vec<$crate::shipments::Shipment>> {
            self.client.execute($crate::shipments::for_picklist(picklist)) $(.$await)?
        }

        /// Lists the ways of shipping that are allowed for a picklist.
        pub $($async)? fn methods(
            &self,
            picklist: $crate::types::PicklistId,
        ) -> $crate::error::Result<Vec<$crate::shipments::ShippingMethod>> {
            self.client.execute($crate::shipments::methods(picklist)) $(.$await)?
        }

        /// Makes a shipment for a picklist and returns it.
        ///
        /// Picqer asks the shipping provider for a label, which can take up to 20 seconds. When all
        /// products of the picklist are picked, the picklist is closed.
        pub $($async)? fn create(
            &self,
            picklist: $crate::types::PicklistId,
            shipment: &$crate::shipments::NewShipment,
        ) -> $crate::error::Result<$crate::shipments::Shipment> {
            self.client.execute($crate::shipments::create(picklist, shipment)?) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use shipments_methods;

/// Operations on shipments.
#[derive(Debug, Clone, Copy)]
pub struct Shipments<'a> {
    pub(crate) client: &'a Client,
}

impl Shipments<'_> {
    shipments_methods!(async; await);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_shipment_with_only_a_profile_sends_only_the_profile() {
        let shipment = NewShipment::new(ShippingProviderProfileId(7));
        assert_eq!(
            serde_json::to_value(&shipment).unwrap(),
            json!({"idshippingprofile": 7})
        );
    }

    #[test]
    fn every_set_part_is_sent() {
        let shipment = NewShipment::new(ShippingProviderProfileId(7))
            .idpackaging(PackagingId(3))
            .weight(Grams::new(900))
            .extrafield("code", "ABC")
            .extrafield("tracking_url", "https://example.com/t/ABC")
            .parcels(NonEmpty::new(vec![NewParcel::new(PackagingId(3), Grams::new(500))]).unwrap());
        assert_eq!(
            serde_json::to_value(&shipment).unwrap(),
            json!({
                "idshippingprofile": 7,
                "idpackaging": 3,
                "weight": 900,
                "extrafields": {"code": "ABC", "tracking_url": "https://example.com/t/ABC"},
                "parcels": [{"idpackaging": 3, "weight": 500}]
            })
        );
    }

    #[test]
    fn the_filters_become_query_pairs() {
        assert!(ShipmentQuery::new().pairs().is_empty());
        let moment = "2026-11-09 17:20:00".parse::<DateTime>().unwrap();
        let query = ShipmentQuery::new()
            .search("ABC")
            .since(moment)
            .until(moment);
        assert_eq!(
            query.pairs(),
            vec![
                ("search", "ABC".to_owned()),
                ("sincedate", "2026-11-09 17:20:00".to_owned()),
                ("untildate", "2026-11-09 17:20:00".to_owned()),
            ]
        );
    }

    #[test]
    fn every_carrier_key_round_trips() {
        for key in CarrierKey::ALL {
            assert_eq!(CarrierKey::from_name(key.as_str()), Some(*key));
        }
        assert_eq!(CarrierKey::ALL.len(), 20);
    }
}
