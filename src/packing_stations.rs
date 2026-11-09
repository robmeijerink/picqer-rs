//! Packing stations.
//!
//! A packing station is a place in the warehouse with printers. The developer account this crate
//! was checked against cannot have any, because Picqer needs PrintNode to make one and does not
//! offer that to developer accounts. These types follow the documentation and have not been
//! compared with real responses, so what the documentation does not mark as always present is
//! optional.

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::reported::Reported;
use crate::types::{PackingStationId, PrinterId};

string_enum! {
    /// The paper format of a printer.
    ///
    /// The documentation shows `a4` and `custom`, so other formats may exist and are kept as
    /// [`Reported::Unknown`].
    pub enum PrinterFormat {
        /// A4 paper.
        A4 = "a4",
        /// A size given by the width and height of the printer.
        Custom = "custom",
    }
}

/// A printer at a packing station.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Printer {
    /// The printer.
    pub idprinter: PrinterId,
    /// The name of the printer.
    pub name: Option<String>,
    /// The paper format.
    pub format: Option<Reported<PrinterFormat>>,
    /// The width of a custom format.
    pub custom_width: Option<i64>,
    /// The height of a custom format.
    pub custom_height: Option<i64>,
    /// Whether the printer understands ZPL.
    pub zpl_support: Option<bool>,
    /// The identifier of the printer at PrintNode.
    pub printnode_printerid: Option<i64>,
}

/// A packing station.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PackingStation {
    /// The packing station.
    pub idpacking_station: PackingStationId,
    /// The name.
    pub name: String,
    /// The printer for packing lists.
    pub printer_packinglists: Option<Printer>,
    /// The printer for shipping labels.
    pub printer_shipping_labels: Option<Printer>,
    /// The printer for shipping documents.
    pub printer_shipping_documents: Option<Printer>,
    /// The printer for product labels.
    pub printer_product_labels: Option<Printer>,
}

pub(crate) fn list(page: Page) -> Call<Vec<PackingStation>> {
    Call::new(page.apply(Request::new(Method::GET, segments(&[&"packingstations"]))))
}

pub(crate) fn get(id: PackingStationId) -> Call<PackingStation> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"packingstations", &id]),
    ))
}

macro_rules! packing_stations_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of packing stations.
        pub $($async)? fn list(
            &self,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::packing_stations::PackingStation>> {
            self.client.execute($crate::packing_stations::list(page)) $(.$await)?
        }

        /// Lists all packing stations, following the pages.
        pub $($async)? fn list_all(
            &self,
        ) -> $crate::error::Result<Vec<$crate::packing_stations::PackingStation>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::packing_stations::list(page)) $(.$await)?)
        }

        /// Fetches a packing station.
        pub $($async)? fn get(
            &self,
            id: $crate::types::PackingStationId,
        ) -> $crate::error::Result<$crate::packing_stations::PackingStation> {
            self.client.execute($crate::packing_stations::get(id)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use packing_stations_methods;

/// Operations on packing stations.
#[derive(Debug, Clone, Copy)]
pub struct PackingStations<'a> {
    pub(crate) client: &'a Client,
}

impl PackingStations<'_> {
    packing_stations_methods!(async; await);
}
