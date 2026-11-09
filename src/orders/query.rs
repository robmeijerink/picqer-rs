use crate::orders::response::OrderStatus;
use crate::types::{
    CustomerId, DateTime, EmailAddress, FulfilmentCustomerId, OrderId, OrderNumber, WarehouseId,
};

/// Filters for listing orders.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrderQuery {
    search: Option<String>,
    since_id: Option<OrderId>,
    before_id: Option<OrderId>,
    since: Option<DateTime>,
    until: Option<DateTime>,
    orderid: Option<OrderNumber>,
    status: Option<OrderStatus>,
    reference: Option<String>,
    emailaddress: Option<EmailAddress>,
    idcustomer: Option<CustomerId>,
    idwarehouse: Option<WarehouseId>,
    idfulfilment_customer: Option<FulfilmentCustomerId>,
    tag: Option<String>,
}

impl OrderQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Searches through the number, reference, customer name and contact name of the orders.
    pub fn search(mut self, text: impl Into<String>) -> Self {
        self.search = Some(text.into());
        self
    }

    /// Lists the orders with a later `idorder` than this one.
    pub fn since_id(mut self, id: OrderId) -> Self {
        self.since_id = Some(id);
        self
    }

    /// Lists the orders with a smaller `idorder` than this one.
    pub fn before_id(mut self, id: OrderId) -> Self {
        self.before_id = Some(id);
        self
    }

    /// Lists the orders added after this moment.
    pub fn since(mut self, moment: DateTime) -> Self {
        self.since = Some(moment);
        self
    }

    /// Lists the orders added before this moment.
    pub fn until(mut self, moment: DateTime) -> Self {
        self.until = Some(moment);
        self
    }

    /// Lists the order with this number.
    pub fn orderid(mut self, orderid: OrderNumber) -> Self {
        self.orderid = Some(orderid);
        self
    }

    /// Lists the orders with this status.
    pub fn status(mut self, status: OrderStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Lists the orders with this reference.
    pub fn reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }

    /// Lists the orders with this email address.
    pub fn emailaddress(mut self, emailaddress: EmailAddress) -> Self {
        self.emailaddress = Some(emailaddress);
        self
    }

    /// Lists the orders of a customer.
    pub fn idcustomer(mut self, idcustomer: CustomerId) -> Self {
        self.idcustomer = Some(idcustomer);
        self
    }

    /// Lists the orders that can be delivered from a warehouse.
    pub fn idwarehouse(mut self, idwarehouse: WarehouseId) -> Self {
        self.idwarehouse = Some(idwarehouse);
        self
    }

    /// Lists the orders of a fulfilment customer, with Picqer Fulfilment.
    pub fn idfulfilment_customer(mut self, customer: FulfilmentCustomerId) -> Self {
        self.idfulfilment_customer = Some(customer);
        self
    }

    /// Lists the orders that have the tag with this name.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    pub(crate) fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        let mut push = |name: &'static str, value: Option<String>| {
            if let Some(value) = value {
                pairs.push((name, value));
            }
        };
        push("search", self.search.clone());
        push("sinceid", self.since_id.map(|id| id.to_string()));
        push("beforeid", self.before_id.map(|id| id.to_string()));
        push("sincedate", self.since.map(|moment| moment.to_string()));
        push("untildate", self.until.map(|moment| moment.to_string()));
        push("orderid", self.orderid.as_ref().map(ToString::to_string));
        push(
            "status",
            self.status.map(|status| status.as_str().to_owned()),
        );
        push("reference", self.reference.clone());
        push(
            "emailaddress",
            self.emailaddress.as_ref().map(ToString::to_string),
        );
        push("idcustomer", self.idcustomer.map(|id| id.to_string()));
        push("idwarehouse", self.idwarehouse.map(|id| id.to_string()));
        push(
            "idfulfilment_customer",
            self.idfulfilment_customer.map(|id| id.to_string()),
        );
        push("tag", self.tag.clone());
        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_filters_send_no_parameters() {
        assert!(OrderQuery::new().pairs().is_empty());
    }

    #[test]
    fn filters_use_their_wire_names_and_formats() {
        let moment: DateTime = "2026-11-01 00:00:00".parse().unwrap();
        let query = OrderQuery::new()
            .status(OrderStatus::Processing)
            .since_id(OrderId(10))
            .since(moment)
            .idcustomer(CustomerId(4))
            .tag("summer");
        assert_eq!(
            query.pairs(),
            vec![
                ("sinceid", "10".to_owned()),
                ("sincedate", "2026-11-01 00:00:00".to_owned()),
                ("status", "processing".to_owned()),
                ("idcustomer", "4".to_owned()),
                ("tag", "summer".to_owned()),
            ]
        );
    }
}
