#![feature(proc_macro_hygiene)]
extern crate hdk;
extern crate hdk_proc_macros;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use hdk::holochain_core_types::entry::Entry;
use hdk::holochain_persistence_api::cas::content::Address;
use hdk::prelude::*;
use hdk::{entry_definition::ValidatingEntryType, error::ZomeApiResult, AGENT_ADDRESS};
use hdk_proc_macros::zome;

// see https://developer.holochain.org/api/0.0.40-alpha1/hdk/ for info on using the hdk library

pub mod badge;
pub mod badge_class;
pub mod utils;

use badge_class::BadgeClass;

#[zome]
mod my_zome {

    #[init]
    fn init() {
        Ok(())
    }

    #[validate_agent]
    pub fn validate_agent(validation_data: EntryValidationData<AgentId>) {
        Ok(())
    }

    #[entry_def]
    fn anchor() -> ValidatingEntryType {
        badge_class::anchor_entry_def()
    }

    #[entry_def]
    fn badge_class() -> ValidatingEntryType {
        badge_class::entry_def()
    }

    #[entry_def]
    fn badge() -> ValidatingEntryType {
        badge::entry_def()
    }

    #[zome_fn("hc_public")]
    fn get_entry(address: Address) -> ZomeApiResult<Option<Entry>> {
        hdk::get_entry(&address)
    }

    #[zome_fn("hc_public")]
    fn get_all_badge_classes() -> ZomeApiResult<Vec<ZomeApiResult<Entry>>> {
        hdk::get_links_and_load(
            &anchor_address()?,
            LinkMatch::Exactly("anchor->badge_class"),
            LinkMatch::Any,
        )
    }

    #[zome_fn("hc_public")]
    fn get_badges_for_class(badge_class: Address) -> ZomeApiResult<Vec<ZomeApiResult<Entry>>> {
        hdk::get_links_and_load(
            &badge_class,
            LinkMatch::Exactly("badge_class->badge"),
            LinkMatch::Any,
        )
    }

    #[zome_fn("hc_public")]
    fn get_badges_to_recipient(agent_address: Address) -> ZomeApiResult<Vec<ZomeApiResult<Entry>>> {
        hdk::get_links_and_load(
            &agent_address,
            LinkMatch::Exactly("recipient->badge"),
            LinkMatch::Any,
        )
    }

    #[zome_fn("hc_public")]
    fn get_badges_from_issuer(agent_address: Address) -> ZomeApiResult<Vec<ZomeApiResult<Entry>>> {
        hdk::get_links_and_load(
            &agent_address,
            LinkMatch::Exactly("issuer->badge"),
            LinkMatch::Any,
        )
    }

    #[zome_fn("hc_public")]
    fn get_created_badges(agent_address: Address) -> ZomeApiResult<Vec<ZomeApiResult<Entry>>> {
        hdk::get_links_and_load(
            &agent_address,
            LinkMatch::Exactly("creator->badge_class"),
            LinkMatch::Any,
        )
    }

    #[zome_fn("hc_public")]
    fn create_badge_class(
        name: String,
        description: String,
        image: String,
        validators: usize,
    ) -> ZomeApiResult<Address> {
        let class = BadgeClass {
            name,
            description,
            image,
            creator_address: AGENT_ADDRESS.clone(),
            validators,
        };

        let class_entry = Entry::App("badge_class".into(), class.into());
        let class_address = hdk::commit_entry(&class_entry)?;

        hdk::link_entries(
            &anchor_address()?,
            &class_address,
            "anchor->badge_class",
            "",
        )?;

        hdk::link_entries(
            &AGENT_ADDRESS.clone(),
            &class_address,
            "creator->badge_class",
            "",
        )?;

        Ok(class_address)
    }

    #[zome_fn("hc_public")]
    fn claim_user_deserves_badge(
        recipient: Address,
        badge_class: Address,
        evidences: Vec<Address>,
    ) -> ZomeApiResult<Address> {
        badge_class::commit_badge_class(&badge_class)?;
        badge::update_badge_with_me_as_issuer(recipient, badge_class, evidences)
    }

    #[zome_fn("hc_public")]
    fn receive_own_badge(badge_class: Address) -> ZomeApiResult<Address> {
        badge_class::commit_badge_class(&badge_class)?;

        let badge = badge::initial_badge(&AGENT_ADDRESS.clone(), &badge_class);
        let badge_address = badge::badge_address(badge)?;

        match hdk::get_entry(&badge_address)? {
            None => Err(ZomeApiError::from(format!(
                "Badge {} not found",
                badge_address
            ))),
            Some(entry) => {
                hdk::debug(format!("hooo, {:?}", entry))?;
                hdk::update_entry(entry, &badge_address)?;
                Ok(badge_address)
            }
        }
    }
}

pub fn anchor_address() -> ZomeApiResult<Address> {
    let entry = Entry::App("anchor".into(), "all_badges_classes".into());

    let anchor_address = hdk::entry_address(&entry)?;

    if let None = hdk::get_entry(&anchor_address)? {
        hdk::commit_entry(&entry)?;
    }

    Ok(anchor_address)
}
