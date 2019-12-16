#![feature(proc_macro_hygiene)]
extern crate hdk;
extern crate hdk_proc_macros;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use hdk::holochain_core_types::{entry::Entry, signature::Provenance, time::Timeout};
use hdk::prelude::*;
use hdk::{entry_definition::ValidatingEntryType, error::ZomeApiResult, AGENT_ADDRESS};

use hdk::holochain_persistence_api::cas::content::Address;

use hdk_proc_macros::zome;

// see https://developer.holochain.org/api/0.0.40-alpha1/hdk/ for info on using the hdk library

pub mod badge_assertion;
pub mod badge_claim;
pub mod badge_class;
pub mod utils;

use badge_assertion::BadgeAssertion;
use badge_claim::BadgeClaim;
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
    fn badge_claim() -> ValidatingEntryType {
        badge_claim::entry_def()
    }

    #[entry_def]
    fn badge_assertion() -> ValidatingEntryType {
        badge_assertion::entry_def()
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
    fn get_badge_class_claims(badge_class: Address) -> ZomeApiResult<Vec<ZomeApiResult<Entry>>> {
        hdk::get_links_and_load(
            &badge_class,
            LinkMatch::Exactly("badge_class->badge_claim"),
            LinkMatch::Any,
        )
    }

    #[zome_fn("hc_public")]
    fn get_badge_class_assertions(
        badge_class: Address,
    ) -> ZomeApiResult<Vec<ZomeApiResult<Entry>>> {
        hdk::get_links_and_load(
            &badge_class,
            LinkMatch::Exactly("badge_class->badge_assertion"),
            LinkMatch::Any,
        )
    }

    #[zome_fn("hc_public")]
    fn get_badge_assertions_to_recipient(
        agent_address: Address,
    ) -> ZomeApiResult<Vec<ZomeApiResult<Entry>>> {
        hdk::get_links_and_load(
            &agent_address,
            LinkMatch::Exactly("recipient->badge_assertion"),
            LinkMatch::Any,
        )
    }

    #[zome_fn("hc_public")]
    fn get_badge_claims_from_issuer(
        agent_address: Address,
    ) -> ZomeApiResult<Vec<ZomeApiResult<Entry>>> {
        hdk::get_links_and_load(
            &agent_address,
            LinkMatch::Exactly("issuer->badge_claim"),
            LinkMatch::Any,
        )
    }

    #[zome_fn("hc_public")]
    fn get_badge_claims_to_recipient(
        agent_address: Address,
    ) -> ZomeApiResult<Vec<ZomeApiResult<Entry>>> {
        hdk::get_links_and_load(
            &agent_address,
            LinkMatch::Exactly("recipient->badge_claim"),
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

        create_badge_assertion(&class_address)?;
        Ok(class_address)
    }

    #[zome_fn("hc_public")]
    fn create_badge_claim(
        recipient: Address,
        badge_class: Address,
        evidences: Vec<Address>,
    ) -> ZomeApiResult<Address> {
        let claim = BadgeClaim {
            issuer: AGENT_ADDRESS.clone(),
            badge_class: badge_class.clone(),
            evidences,
            recipient: recipient.clone(),
        };

        let entry = Entry::App("badge_claim".into(), claim.into());
        let address = hdk::commit_entry(&entry)?;

        hdk::link_entries(
            &AGENT_ADDRESS,
            &address,
            "issuer->badge_claim",
            String::from(badge_class.clone()).as_str(),
        )?;
        hdk::link_entries(
            &recipient,
            &address,
            "recipient->badge_claim",
            String::from(badge_class.clone()).as_str(),
        )?;
        hdk::link_entries(&badge_class, &address, "badge_class->badge_claim", "")?;
        Ok(address)
    }

    #[zome_fn("hc_public")]
    fn create_own_badge_assertion(badge_class: Address) -> ZomeApiResult<Address> {
        let badge_entry: GetEntryResult = hdk::get_entry_result(
            &badge_class,
            GetEntryOptions::new(StatusRequestKind::Initial, true, true, Timeout::default()),
        )?;

        if !badge_entry.found() {
            return Err(ZomeApiError::from(format!(
                "Badge class {} not found",
                badge_class
            )));
        }
        commit_with_provenances(badge_entry)?;
        commit_all_claims(&badge_class)?;
        create_badge_assertion(&badge_class)
    }
}

pub fn commit_all_claims(badge_class: &Address) -> ZomeApiResult<()> {
    let claims: Vec<ZomeApiResult<GetEntryResult>> = hdk::get_links_result(
        &AGENT_ADDRESS,
        LinkMatch::Exactly("recipient->badge_claim"),
        LinkMatch::Exactly(String::from(badge_class.clone()).as_str()),
        GetLinksOptions::default(),
        GetEntryOptions::new(StatusRequestKind::Initial, true, true, Timeout::default()),
    )?;

    for claim in claims {
        let claim_entry = claim?;
        commit_with_provenances(claim_entry)?;
    }

    Ok(())
}

pub fn commit_with_provenances(entry: GetEntryResult) -> ZomeApiResult<()> {
    if let GetEntryResultType::Single(entry_result) = entry.result {
        let entry = entry_result.clone().entry.unwrap();
        let provenances: Vec<Provenance> = entry_result
            .headers
            .iter()
            .flat_map(|header| header.provenances().clone())
            .collect();
        hdk::debug(format!("huhuhu1 {:?}", entry_result))?;
        hdk::debug(format!("huhuhu2 {:?}", provenances))?;
        hdk::commit_entry_result(&entry, CommitEntryOptions::new(provenances))?;
        Ok(())
    } else {
        return Err(ZomeApiError::from(format!(
            "Badge class get entry had more than one item in its history"
        )));
    }
}

pub fn create_badge_assertion(badge_class: &Address) -> ZomeApiResult<Address> {
    let assertion = BadgeAssertion {
        recipient: AGENT_ADDRESS.clone(),
        badge_class: badge_class.clone(),
    };

    let entry = Entry::App("badge_assertion".into(), assertion.into());
    let address = hdk::commit_entry(&entry)?;

    hdk::link_entries(&AGENT_ADDRESS, &address, "recipient->badge_assertion", "")?;

    hdk::link_entries(badge_class, &address, "badge_class->badge_assertion", "")?;

    Ok(address)
}

pub fn anchor_address() -> ZomeApiResult<Address> {
    let entry = Entry::App("anchor".into(), "all_badges_classes".into());

    let anchor_address = hdk::entry_address(&entry)?;

    if let None = hdk::get_entry(&anchor_address)? {
        hdk::commit_entry(&entry)?;
    }

    Ok(anchor_address)
}
