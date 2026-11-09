use crate::picklists::response::PicklistStatus;
use crate::types::{
    DateTime, FulfilmentCustomerId, PicklistId, PicklistNumber, UserId, WarehouseId,
};

/// Filters for listing picklists.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PicklistQuery {
    since_id: Option<PicklistId>,
    before_id: Option<PicklistId>,
    since: Option<DateTime>,
    until: Option<DateTime>,
    closed_after: Option<DateTime>,
    closed_before: Option<DateTime>,
    picklistid: Option<PicklistNumber>,
    assigned_to: Option<UserId>,
    idwarehouse: Option<WarehouseId>,
    status: Option<PicklistStatus>,
    idfulfilment_customer: Option<FulfilmentCustomerId>,
    tag: Option<String>,
}

impl PicklistQuery {
    /// No filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lists the picklists with a later `idpicklist` than this one.
    pub fn since_id(mut self, id: PicklistId) -> Self {
        self.since_id = Some(id);
        self
    }

    /// Lists the picklists with a smaller `idpicklist` than this one.
    pub fn before_id(mut self, id: PicklistId) -> Self {
        self.before_id = Some(id);
        self
    }

    /// Lists the picklists created after this moment.
    pub fn since(mut self, moment: DateTime) -> Self {
        self.since = Some(moment);
        self
    }

    /// Lists the picklists created before this moment.
    pub fn until(mut self, moment: DateTime) -> Self {
        self.until = Some(moment);
        self
    }

    /// Lists the picklists closed after this moment.
    pub fn closed_after(mut self, moment: DateTime) -> Self {
        self.closed_after = Some(moment);
        self
    }

    /// Lists the picklists closed before this moment.
    pub fn closed_before(mut self, moment: DateTime) -> Self {
        self.closed_before = Some(moment);
        self
    }

    /// Lists the picklist with this number.
    pub fn picklistid(mut self, picklistid: PicklistNumber) -> Self {
        self.picklistid = Some(picklistid);
        self
    }

    /// Lists the picklists assigned to a user.
    pub fn assigned_to(mut self, user: UserId) -> Self {
        self.assigned_to = Some(user);
        self
    }

    /// Lists the picklists of a warehouse.
    pub fn idwarehouse(mut self, idwarehouse: WarehouseId) -> Self {
        self.idwarehouse = Some(idwarehouse);
        self
    }

    /// Lists the picklists with this status.
    pub fn status(mut self, status: PicklistStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Lists the picklists of a fulfilment customer, with Picqer Fulfilment.
    pub fn idfulfilment_customer(mut self, customer: FulfilmentCustomerId) -> Self {
        self.idfulfilment_customer = Some(customer);
        self
    }

    /// Lists the picklists that have the tag with this name.
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
        push("sinceid", self.since_id.map(|id| id.to_string()));
        push("beforeid", self.before_id.map(|id| id.to_string()));
        push("sincedate", self.since.map(|moment| moment.to_string()));
        push("untildate", self.until.map(|moment| moment.to_string()));
        push(
            "closed_after",
            self.closed_after.map(|moment| moment.to_string()),
        );
        push(
            "closed_before",
            self.closed_before.map(|moment| moment.to_string()),
        );
        push(
            "picklistid",
            self.picklistid.as_ref().map(ToString::to_string),
        );
        push(
            "assigned_to_iduser",
            self.assigned_to.map(|user| user.to_string()),
        );
        push("idwarehouse", self.idwarehouse.map(|id| id.to_string()));
        push(
            "status",
            self.status.map(|status| status.as_str().to_owned()),
        );
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
    fn filters_use_their_wire_names() {
        let query = PicklistQuery::new()
            .status(PicklistStatus::Snoozed)
            .assigned_to(UserId(3))
            .since_id(PicklistId(9));
        assert_eq!(
            query.pairs(),
            vec![
                ("sinceid", "9".to_owned()),
                ("assigned_to_iduser", "3".to_owned()),
                ("status", "snoozed".to_owned()),
            ]
        );
        assert!(PicklistQuery::new().pairs().is_empty());
    }
}
