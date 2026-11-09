//! Presets to create picklist batches with.
//!
//! Presets are made in the Picqer app. They hold the filters and the settings of a batch, and a batch
//! made from one gets its title and its assigned user.

use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::http::{Request, segments};
use crate::ops::Call;
use crate::pagination::Page;
use crate::types::PicklistBatchPresetId;

/// Which presets to list.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PresetStatus {
    /// The presets that are in use.
    #[default]
    Active,
    /// The presets that are not used in wizards or forms in Picqer anymore.
    Inactive,
}

/// A preset to create picklist batches with.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PicklistBatchPreset {
    /// The preset.
    pub idpicklist_batch_preset: PicklistBatchPresetId,
    /// The name of the preset.
    pub name: Option<String>,
    /// Whether the preset is active.
    pub active: Option<bool>,
    /// How many picklists match the filters of the preset. Only a single preset has it.
    pub picklists_count: Option<i64>,
}

pub(crate) fn list(status: PresetStatus, page: Page) -> Call<Vec<PicklistBatchPreset>> {
    let mut request = Request::new(
        Method::GET,
        segments(&[&"picklists", &"batches", &"presets"]),
    );
    if status == PresetStatus::Inactive {
        request = request.query("inactive", "true");
    }
    Call::new(page.apply(request))
}

pub(crate) fn get(id: PicklistBatchPresetId) -> Call<PicklistBatchPreset> {
    Call::new(Request::new(
        Method::GET,
        segments(&[&"picklists", &"batches", &"presets", &id]),
    ))
}

macro_rules! picklist_batch_presets_methods {
    ($($async:ident)?; $($await:ident)?) => {
        /// Lists one page of the active or of the inactive presets.
        pub $($async)? fn list(
            &self,
            status: $crate::picklist_batch_presets::PresetStatus,
            page: $crate::pagination::Page,
        ) -> $crate::error::Result<Vec<$crate::picklist_batch_presets::PicklistBatchPreset>> {
            self.client.execute($crate::picklist_batch_presets::list(status, page)) $(.$await)?
        }

        /// Lists all active or all inactive presets, following the pages.
        pub $($async)? fn list_all(
            &self,
            status: $crate::picklist_batch_presets::PresetStatus,
        ) -> $crate::error::Result<Vec<$crate::picklist_batch_presets::PicklistBatchPreset>> {
            $crate::pagination::collect_pages!(page => self.client.execute($crate::picklist_batch_presets::list(status, page)) $(.$await)?)
        }

        /// Fetches a preset, active or not. Only a single preset says how many picklists match it.
        pub $($async)? fn get(
            &self,
            id: $crate::types::PicklistBatchPresetId,
        ) -> $crate::error::Result<$crate::picklist_batch_presets::PicklistBatchPreset> {
            self.client.execute($crate::picklist_batch_presets::get(id)) $(.$await)?
        }
    };
}

#[cfg(feature = "blocking")]
pub(crate) use picklist_batch_presets_methods;

/// Operations on the presets of picklist batches.
#[derive(Debug, Clone, Copy)]
pub struct PicklistBatchPresets<'a> {
    pub(crate) client: &'a Client,
}

impl PicklistBatchPresets<'_> {
    picklist_batch_presets_methods!(async; await);
}
