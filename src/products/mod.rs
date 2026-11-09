//! Products.

mod history;
mod images;
mod locations;
mod parts;
mod query;
mod request;
mod response;
mod settings;

pub use history::{ExpectedDelivery, FixedStockPriceChange, PriceChange};
pub use images::{ImageType, NewProductImage, ProductImage};
pub use locations::{
    LotNumber, NewProductLocation, ProductLocation, ProductLocationChanges, ProductLocationStock,
};
pub(crate) use locations::{
    link_location, locations, move_stock, unlink_location, update_location,
};
pub use parts::{CompositionTree, NewProductPart, PartTree, ProductPart, Production};
pub use query::{ProductQuery, ProductStatus};
pub use request::{NewProduct, ProductChanges, ProductFieldValue};
pub use response::{AbcClassification, Product, ProductFieldEntry, ProductSupplier, ProductType};
pub use settings::{WarehouseSettings, WarehouseSettingsChanges};

use std::num::NonZeroU32;

use reqwest::Method;
use serde_json::json;

use crate::client::Client;
use crate::error::Result;
use crate::http::{Request, segments};
use crate::ops::{Call, NoContent};
use crate::pagination::Page;
use crate::stock::{StockLevel, StockUpdate, WarehouseStockDetail};
use crate::tags::Tag;
use crate::types::{PricelistId, ProductId, ProductImageId, ProductPartId, TagId, WarehouseId};

pub(crate) fn list(query: &ProductQuery, page: Page) -> Call<Vec<Product>> {
    let mut request = Request::new(Method::GET, segments(&[&"products"]));
    for (name, value) in query.pairs() {
        request = request.query(name, value);
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: ProductId) -> Call<Product> {
    Call::new(Request::new(Method::GET, segments(&[&"products", &id])))
}

pub(crate) fn create(product: &NewProduct) -> Result<Call<Product>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"products"])).json_body(product)?,
    ))
}

pub(crate) fn update(id: ProductId, changes: &ProductChanges) -> Result<Call<Product>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"products", &id])).json_body(changes)?,
    ))
}

pub(crate) fn set_active(id: ProductId, active: bool) -> Call<Product> {
    let action = if active { "activate" } else { "inactivate" };
    Call::new(Request::new(
        Method::POST,
        segments(&[&"products", &id, &action]),
    ))
}

pub(crate) fn stock(id: ProductId) -> Call<Vec<StockLevel>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"stock"]),
    ))
}

pub(crate) fn stock_in_warehouse(
    id: ProductId,
    warehouse: WarehouseId,
) -> Call<WarehouseStockDetail> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"stock", &warehouse]),
    ))
}

pub(crate) fn update_stock(
    id: ProductId,
    warehouse: WarehouseId,
    update: &StockUpdate,
) -> Result<Call<WarehouseStockDetail>> {
    Ok(Call::new(
        Request::new(
            Method::POST,
            segments(&[&"products", &id, &"stock", &warehouse]),
        )
        .json_body(update)?,
    ))
}

fn nested(request: Request) -> Request {
    request.query("nested", "true")
}

pub(crate) fn parts(id: ProductId) -> Call<Vec<ProductPart>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"parts"]),
    ))
}

pub(crate) fn parts_tree(id: ProductId) -> Call<Vec<PartTree>> {
    Call::new(nested(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"parts"]),
    )))
}

pub(crate) fn part(id: ProductId, part: ProductPartId) -> Call<ProductPart> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"parts", &part]),
    ))
}

pub(crate) fn add_part(id: ProductId, part: &NewProductPart) -> Result<Call<ProductPart>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"products", &id, &"parts"])).json_body(part)?,
    ))
}

pub(crate) fn update_part(
    id: ProductId,
    part: ProductPartId,
    amount: NonZeroU32,
) -> Result<Call<ProductPart>> {
    Ok(Call::new(
        Request::new(Method::PUT, segments(&[&"products", &id, &"parts", &part]))
            .json_body(&json!({"amount": amount}))?,
    ))
}

pub(crate) fn remove_part(id: ProductId, part: ProductPartId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"products", &id, &"parts", &part]),
    ))
}

pub(crate) fn compositions(id: ProductId, page: Page) -> Call<Vec<ProductPart>> {
    Call::new(page.apply(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"compositions"]),
    )))
}

pub(crate) fn compositions_tree(id: ProductId, page: Page) -> Call<Vec<CompositionTree>> {
    Call::new(page.apply(nested(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"compositions"]),
    ))))
}

pub(crate) fn produce(id: ProductId, production: &Production) -> Result<NoContent> {
    Ok(NoContent::new(
        Request::new(Method::POST, segments(&[&"products", &id, &"produce"]))
            .json_body(production)?,
    ))
}

pub(crate) fn warehouse_settings(id: ProductId) -> Call<Vec<WarehouseSettings>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"warehouses"]),
    ))
}

pub(crate) fn update_warehouse_settings(
    id: ProductId,
    warehouse: WarehouseId,
    changes: &WarehouseSettingsChanges,
) -> Result<Call<WarehouseSettings>> {
    Ok(Call::new(
        Request::new(
            Method::PUT,
            segments(&[&"products", &id, &"warehouses", &warehouse]),
        )
        .json_body(changes)?,
    ))
}

pub(crate) fn images(id: ProductId, page: Page) -> Call<Vec<ProductImage>> {
    Call::new(page.apply(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"images"]),
    )))
}

pub(crate) fn add_image(id: ProductId, image: &NewProductImage) -> Result<Call<ProductImage>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"products", &id, &"images"])).json_body(image)?,
    ))
}

pub(crate) fn remove_image(id: ProductId, image: ProductImageId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"products", &id, &"images", &image]),
    ))
}

pub(crate) fn price_history(id: ProductId) -> Call<Vec<PriceChange>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"pricehistory"]),
    ))
}

pub(crate) fn fixed_stock_price_history(id: ProductId) -> Call<Vec<FixedStockPriceChange>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"fixedstockpricehistory"]),
    ))
}

pub(crate) fn expected(id: ProductId) -> Call<Vec<ExpectedDelivery>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"expected"]),
    ))
}

pub(crate) fn tags(id: ProductId) -> Call<Vec<Tag>> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"products", &id, &"tags"]),
    ))
}

pub(crate) fn add_tag(id: ProductId, tag: TagId) -> Result<Call<Tag>> {
    Ok(Call::new(
        Request::new(Method::POST, segments(&[&"products", &id, &"tags"]))
            .json_body(&json!({"idtag": tag}))?,
    ))
}

pub(crate) fn remove_tag(id: ProductId, tag: TagId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"products", &id, &"tags", &tag]),
    ))
}

pub(crate) fn remove_pricelist_price(id: ProductId, pricelist: PricelistId) -> NoContent {
    NoContent::new(Request::new(
        Method::DELETE,
        segments(&[&"products", &id, &"pricelists", &pricelist]),
    ))
}

macro_rules! products_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of products.
        pub $($async)? fn list(
            &self,
            query: &$crate::products::ProductQuery,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::products::Product>> {
            self.client.execute($crate::products::list(query, page)) $(.$await)?
        }

        /// Lists all products that match a query, following the pages.
        pub $($async)? fn list_all(
            &self,
            query: &$crate::products::ProductQuery,
        ) -> $crate::error::Result<Vec<$crate::products::Product>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::products::list(query, page)) $(.$await)?)
        }

        /// Fetches a product.
        pub $($async)? fn get(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<$crate::products::Product> {
            self.client.execute($crate::products::get(id)) $(.$await)?
        }

        /// Finds an active or inactive product by its code.
        pub $($async)? fn find_by_productcode(
            &self,
            productcode: &$crate::types::ProductCode,
        ) -> $crate::error::Result<Option<$crate::products::Product>> {
            let mut found = Vec::new();
            for status in [$crate::products::ProductStatus::Active, $crate::products::ProductStatus::Inactive] {
                let query = $crate::products::ProductQuery::new()
                    .status(status)
                    .productcode(productcode.clone());
                found = self.client.execute($crate::products::list(&query, $crate::pagination::Page::FIRST)) $(.$await)? ?;
                if !found.is_empty() {
                    break;
                }
            }
            Ok(found.into_iter().next())
        }

        /// Creates a product.
        pub $($async)? fn create(
            &self,
            product: &$crate::products::NewProduct,
        ) -> $crate::error::Result<$crate::products::Product> {
            self.client.execute($crate::products::create(product)?) $(.$await)?
        }

        /// Changes a product.
        pub $($async)? fn update(
            &self,
            id: $crate::types::ProductId,
            changes: &$crate::products::ProductChanges,
        ) -> $crate::error::Result<$crate::products::Product> {
            self.client.execute($crate::products::update(id, changes)?) $(.$await)?
        }

        /// Makes a product active again.
        pub $($async)? fn activate(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<$crate::products::Product> {
            self.client.execute($crate::products::set_active(id, true)) $(.$await)?
        }

        /// Makes a product inactive. A product can only be made inactive when it has no stock, is
        /// in no compositions and is on no open picklists, backorders or purchase orders.
        pub $($async)? fn inactivate(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<$crate::products::Product> {
            self.client.execute($crate::products::set_active(id, false)) $(.$await)?
        }

        /// Lists the stock of a product per warehouse.
        pub $($async)? fn stock(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<Vec<$crate::stock::StockLevel>> {
            self.client.execute($crate::products::stock(id)) $(.$await)?
        }

        /// Fetches the stock of a product in one warehouse, with the stock per location.
        pub $($async)? fn stock_in_warehouse(
            &self,
            id: $crate::types::ProductId,
            warehouse: $crate::types::WarehouseId,
        ) -> $crate::error::Result<$crate::stock::WarehouseStockDetail> {
            self.client.execute($crate::products::stock_in_warehouse(id, warehouse)) $(.$await)?
        }

        /// Changes the stock of a product in a warehouse and returns the stock afterwards.
        pub $($async)? fn update_stock(
            &self,
            id: $crate::types::ProductId,
            warehouse: $crate::types::WarehouseId,
            update: &$crate::stock::StockUpdate,
        ) -> $crate::error::Result<$crate::stock::WarehouseStockDetail> {
            self.client.execute($crate::products::update_stock(id, warehouse, update)?) $(.$await)?
        }

        /// Lists the parts of a composition. Picqer answers a product that is not a composition
        /// with a not found error.
        pub $($async)? fn parts(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<Vec<$crate::products::ProductPart>> {
            self.client.execute($crate::products::parts(id)) $(.$await)?
        }

        /// Lists the parts of a composition with the parts of the virtual compositions among
        /// them, down the whole tree.
        pub $($async)? fn parts_tree(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<Vec<$crate::products::PartTree>> {
            self.client.execute($crate::products::parts_tree(id)) $(.$await)?
        }

        /// Fetches one part of a composition.
        pub $($async)? fn part(
            &self,
            id: $crate::types::ProductId,
            part: $crate::types::ProductPartId,
        ) -> $crate::error::Result<$crate::products::ProductPart> {
            self.client.execute($crate::products::part(id, part)) $(.$await)?
        }

        /// Adds a product to a composition.
        pub $($async)? fn add_part(
            &self,
            id: $crate::types::ProductId,
            part: &$crate::products::NewProductPart,
        ) -> $crate::error::Result<$crate::products::ProductPart> {
            self.client.execute($crate::products::add_part(id, part)?) $(.$await)?
        }

        /// Changes how many of a part a composition holds.
        pub $($async)? fn update_part(
            &self,
            id: $crate::types::ProductId,
            part: $crate::types::ProductPartId,
            amount: ::std::num::NonZeroU32,
        ) -> $crate::error::Result<$crate::products::ProductPart> {
            self.client.execute($crate::products::update_part(id, part, amount)?) $(.$await)?
        }

        /// Takes a part out of a composition.
        pub $($async)? fn remove_part(
            &self,
            id: $crate::types::ProductId,
            part: $crate::types::ProductPartId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::products::remove_part(id, part)) $(.$await)?
        }

        /// Lists one page of the compositions a product is part of.
        pub $($async)? fn compositions(
            &self,
            id: $crate::types::ProductId,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::products::ProductPart>> {
            self.client.execute($crate::products::compositions(id, page)) $(.$await)?
        }

        /// Lists all the compositions a product is part of, following the pages.
        pub $($async)? fn compositions_all(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<Vec<$crate::products::ProductPart>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::products::compositions(id, page)) $(.$await)?)
        }

        /// Lists one page of the compositions a product is part of, with the compositions those
        /// are part of when they are virtual.
        pub $($async)? fn compositions_tree(
            &self,
            id: $crate::types::ProductId,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::products::CompositionTree>> {
            self.client.execute($crate::products::compositions_tree(id, page)) $(.$await)?
        }

        /// Lists all the compositions a product is part of, with the compositions those are part
        /// of when they are virtual, following the pages.
        pub $($async)? fn compositions_tree_all(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<Vec<$crate::products::CompositionTree>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::products::compositions_tree(id, page)) $(.$await)?)
        }

        /// Registers the production of a composition that has stock of its own.
        pub $($async)? fn produce(
            &self,
            id: $crate::types::ProductId,
            production: &$crate::products::Production,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::products::produce(id, production)?) $(.$await)?
        }

        /// Lists the settings of a product per warehouse.
        pub $($async)? fn warehouse_settings(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<Vec<$crate::products::WarehouseSettings>> {
            self.client.execute($crate::products::warehouse_settings(id)) $(.$await)?
        }

        /// Changes the settings of a product in a warehouse and returns the settings afterwards.
        pub $($async)? fn update_warehouse_settings(
            &self,
            id: $crate::types::ProductId,
            warehouse: $crate::types::WarehouseId,
            changes: &$crate::products::WarehouseSettingsChanges,
        ) -> $crate::error::Result<$crate::products::WarehouseSettings> {
            self.client.execute($crate::products::update_warehouse_settings(id, warehouse, changes)?) $(.$await)?
        }

        /// Lists one page of the images of a product.
        pub $($async)? fn images(
            &self,
            id: $crate::types::ProductId,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::products::ProductImage>> {
            self.client.execute($crate::products::images(id, page)) $(.$await)?
        }

        /// Lists all the images of a product, following the pages.
        pub $($async)? fn images_all(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<Vec<$crate::products::ProductImage>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::products::images(id, page)) $(.$await)?)
        }

        /// Adds an image to a product.
        pub $($async)? fn add_image(
            &self,
            id: $crate::types::ProductId,
            image: &$crate::products::NewProductImage,
        ) -> $crate::error::Result<$crate::products::ProductImage> {
            self.client.execute($crate::products::add_image(id, image)?) $(.$await)?
        }

        /// Deletes an image of a product.
        pub $($async)? fn remove_image(
            &self,
            id: $crate::types::ProductId,
            image: $crate::types::ProductImageId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::products::remove_image(id, image)) $(.$await)?
        }

        /// Lists the changes of the price of a product.
        pub $($async)? fn price_history(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<Vec<$crate::products::PriceChange>> {
            self.client.execute($crate::products::price_history(id)) $(.$await)?
        }

        /// Lists the changes of the fixed stock price of a product.
        pub $($async)? fn fixed_stock_price_history(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<Vec<$crate::products::FixedStockPriceChange>> {
            self.client.execute($crate::products::fixed_stock_price_history(id)) $(.$await)?
        }

        /// Lists the lines of outstanding purchase orders for a product, in order of expected
        /// arrival.
        pub $($async)? fn expected(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<Vec<$crate::products::ExpectedDelivery>> {
            self.client.execute($crate::products::expected(id)) $(.$await)?
        }

        /// Lists the locations a product is linked to, with its stock at each.
        pub $($async)? fn locations(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<Vec<$crate::products::ProductLocation>> {
            self.client.execute($crate::products::locations(id)) $(.$await)?
        }

        /// Links a location to a product.
        pub $($async)? fn link_location(
            &self,
            id: $crate::types::ProductId,
            location: &$crate::products::NewProductLocation,
        ) -> $crate::error::Result<$crate::products::ProductLocation> {
            self.client.execute($crate::products::link_location(id, location)?) $(.$await)?
        }

        /// Changes the note, the preference and the lot details of a linked location.
        pub $($async)? fn update_location(
            &self,
            id: $crate::types::ProductId,
            location: $crate::types::LocationId,
            changes: &$crate::products::ProductLocationChanges,
        ) -> $crate::error::Result<$crate::products::ProductLocation> {
            self.client.execute($crate::products::update_location(id, location, changes)?) $(.$await)?
        }

        /// Unlinks a location from a product, which removes its note and preference. Picqer refuses a
        /// location that still has stock of the product.
        pub $($async)? fn unlink_location(
            &self,
            id: $crate::types::ProductId,
            location: $crate::types::LocationId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::products::unlink_location(id, location)) $(.$await)?
        }

        /// Moves stock of a product between locations or warehouses. A location that was not
        /// linked to the product is linked.
        pub $($async)? fn move_stock(
            &self,
            id: $crate::types::ProductId,
            movement: &$crate::stock::StockMove,
        ) -> $crate::error::Result<$crate::stock::StockMoveResult> {
            self.client.execute($crate::products::move_stock(id, movement)?) $(.$await)?
        }

        /// Lists the tags of a product.
        pub $($async)? fn tags(
            &self,
            id: $crate::types::ProductId,
        ) -> $crate::error::Result<Vec<$crate::tags::Tag>> {
            self.client.execute($crate::products::tags(id)) $(.$await)?
        }

        /// Puts a tag on a product.
        pub $($async)? fn add_tag(
            &self,
            id: $crate::types::ProductId,
            tag: $crate::types::TagId,
        ) -> $crate::error::Result<$crate::tags::Tag> {
            self.client.execute($crate::products::add_tag(id, tag)?) $(.$await)?
        }

        /// Takes a tag off a product.
        pub $($async)? fn remove_tag(
            &self,
            id: $crate::types::ProductId,
            tag: $crate::types::TagId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::products::remove_tag(id, tag)) $(.$await)?
        }

        /// Removes the price of a product in a pricelist.
        pub $($async)? fn remove_pricelist_price(
            &self,
            id: $crate::types::ProductId,
            pricelist: $crate::types::PricelistId,
        ) -> $crate::error::Result<()> {
            self.client.execute($crate::products::remove_pricelist_price(id, pricelist)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use products_methods;

/// Operations on products.
#[derive(Debug, Clone, Copy)]
pub struct Products<'a> {
    pub(crate) client: &'a Client,
}

impl Products<'_> {
    products_methods!(async; await);
}
