use crate::products::response::ProductType;
use crate::types::{FulfilmentCustomerId, NonEmpty, ProductCode, ProductId, SupplierId};

/// Which products to list. Picqer lists the active ones unless asked for the inactive ones.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProductStatus {
    /// The active products.
    #[default]
    Active,
    /// The inactive products.
    Inactive,
}

/// Filters for listing products.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProductQuery {
    status: ProductStatus,
    tag: Option<String>,
    product_type: Option<ProductType>,
    productcode: Option<ProductCode>,
    idsupplier: Option<SupplierId>,
    idproducts: Option<NonEmpty<ProductId, 100>>,
    search: Option<String>,
    idfulfilment_customer: Option<FulfilmentCustomerId>,
}

impl ProductQuery {
    /// No filters: the active products of every type.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists the active or the inactive products.
    ///
    /// Picqer treats the presence of the parameter as asking for the inactive products, whatever
    /// its value, so the active products are asked for by leaving it out.
    pub fn status(mut self, status: ProductStatus) -> Self {
        self.status = status;
        self
    }

    /// Lists the products that have the tag with this name.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Lists the products of one type.
    pub fn product_type(mut self, product_type: ProductType) -> Self {
        self.product_type = Some(product_type);
        self
    }

    /// Lists the product with this code. Use it to find one product.
    pub fn productcode(mut self, productcode: ProductCode) -> Self {
        self.productcode = Some(productcode);
        self
    }

    /// Lists the products linked to a supplier.
    pub fn idsupplier(mut self, idsupplier: SupplierId) -> Self {
        self.idsupplier = Some(idsupplier);
        self
    }

    /// Lists specific products, at most 100 at a time.
    pub fn idproducts(mut self, idproducts: NonEmpty<ProductId, 100>) -> Self {
        self.idproducts = Some(idproducts);
        self
    }

    /// Searches approximately through the name, product code, supplier product code and barcode.
    pub fn search(mut self, text: impl Into<String>) -> Self {
        self.search = Some(text.into());
        self
    }

    /// Lists the products of a fulfilment customer, with Picqer Fulfilment.
    pub fn idfulfilment_customer(mut self, customer: FulfilmentCustomerId) -> Self {
        self.idfulfilment_customer = Some(customer);
        self
    }

    pub(crate) fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if self.status == ProductStatus::Inactive {
            pairs.push(("inactive", "true".to_owned()));
        }
        if let Some(tag) = &self.tag {
            pairs.push(("tag", tag.clone()));
        }
        if let Some(product_type) = self.product_type {
            pairs.push(("type", product_type.as_str().to_owned()));
        }
        if let Some(productcode) = &self.productcode {
            pairs.push(("productcode", productcode.to_string()));
        }
        if let Some(idsupplier) = self.idsupplier {
            pairs.push(("idsupplier", idsupplier.to_string()));
        }
        if let Some(idproducts) = &self.idproducts {
            let joined: Vec<String> = idproducts.iter().map(ToString::to_string).collect();
            pairs.push(("idproducts", joined.join(",")));
        }
        if let Some(search) = &self.search {
            pairs.push(("search", search.clone()));
        }
        if let Some(customer) = self.idfulfilment_customer {
            pairs.push(("idfulfilment_customer", customer.to_string()));
        }
        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_filters_send_no_parameters() {
        assert!(ProductQuery::new().pairs().is_empty());
    }

    #[test]
    fn the_active_status_is_never_sent() {
        let query = ProductQuery::new().status(ProductStatus::Active);
        assert!(query.pairs().is_empty());
        let query = ProductQuery::new().status(ProductStatus::Inactive);
        assert_eq!(query.pairs(), vec![("inactive", "true".to_owned())]);
    }

    #[test]
    fn filters_use_their_wire_names() {
        let query = ProductQuery::new()
            .product_type(ProductType::VirtualComposition)
            .idproducts(NonEmpty::new(vec![ProductId(1), ProductId(22)]).unwrap())
            .search("vest");
        assert_eq!(
            query.pairs(),
            vec![
                ("type", "virtual_composition".to_owned()),
                ("idproducts", "1,22".to_owned()),
                ("search", "vest".to_owned()),
            ]
        );
    }
}
