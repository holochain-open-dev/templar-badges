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
pub mod anchor;

use badge::Badge;
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
        anchor::entry_def()
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
    fn get_my_address() -> ZomeApiResult<Address> {
        Ok(AGENT_ADDRESS.clone())
    }

    #[zome_fn("hc_public")]
    fn get_badge(recipient: Address, badge_class: Address) -> ZomeApiResult<Option<Entry>> {
        let badge = Badge::initial(&recipient, &badge_class);
        hdk::get_entry(&badge.address()?)
    }

    #[zome_fn("hc_public")]
    fn get_entry_history(address: Address) -> ZomeApiResult<Option<EntryHistory>> {
        hdk::get_entry_history(&address)
    }

    #[zome_fn("hc_public")]
    fn get_all_badge_classes() -> ZomeApiResult<Vec<Address>> {
        let links = hdk::get_links(
            &anchor::address()?,
            LinkMatch::Exactly("anchor->badge_class"),
            LinkMatch::Any,
        )?;

        Ok(links.addresses())
    }

    #[zome_fn("hc_public")]
    fn get_badges_for_class(badge_class: Address) -> ZomeApiResult<Vec<Address>> {
        let links = hdk::get_links(
            &badge_class,
            LinkMatch::Exactly("badge_class->badge"),
            LinkMatch::Any,
        )?;

        Ok(links.addresses())
    }

    #[zome_fn("hc_public")]
    fn get_badges_to_recipient(agent_address: Address) -> ZomeApiResult<Vec<Address>> {
        let links = hdk::get_links(
            &agent_address,
            LinkMatch::Exactly("recipient->badge"),
            LinkMatch::Exactly("completed"),
        )?;

        Ok(links.addresses())
    }

    #[zome_fn("hc_public")]
    fn get_temptative_badges_to_recipient(agent_address: Address) -> ZomeApiResult<Vec<Address>> {
        let links = hdk::get_links(
            &agent_address,
            LinkMatch::Exactly("recipient->badge"),
            LinkMatch::Exactly("temptative"),
        )?;

        Ok(links.addresses())
    }

    #[zome_fn("hc_public")]
    fn get_badges_from_issuer(agent_address: Address) -> ZomeApiResult<Vec<Address>> {
        let links = hdk::get_links(
            &agent_address,
            LinkMatch::Exactly("issuer->badge"),
            LinkMatch::Any,
        )?;

        Ok(links.addresses())
    }

    #[zome_fn("hc_public")]
    fn get_created_badges(agent_address: Address) -> ZomeApiResult<Vec<Address>> {
        let links = hdk::get_links(
            &agent_address,
            LinkMatch::Exactly("creator->badge_class"),
            LinkMatch::Any,
        )?;

        Ok(links.addresses())
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
            &anchor::address()?,
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
    fn claim_agent_deserves_badge(
        recipient: Address,
        badge_class: Address,
        evidences: Vec<Address>,
    ) -> ZomeApiResult<Address> {
        badge::claim_agent_deserves_badge(recipient, badge_class, evidences)
    }
}
