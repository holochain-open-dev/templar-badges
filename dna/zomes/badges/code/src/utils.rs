use crate::badge_class::BadgeClass;
use hdk::error::ZomeApiError;
use hdk::prelude::*;
use std::convert::TryFrom;

pub fn get_package_entries(package: &hdk::ValidationPackage) -> ZomeApiResult<Vec<Entry>> {
    package
        .source_chain_entries
        .clone()
        .ok_or(ZomeApiError::from(String::from(
            "Validation package must contain source_chain_entries to determine its author",
        )))
}

/**
 * Go through all the agent's chain and find its AgentId entry, and return its agent_address
 */
pub fn get_chain_agent_id(chain_entries: &Vec<Entry>) -> ZomeApiResult<Address> {
    for entry in chain_entries.iter() {
        if let Entry::AgentId(agent_id) = entry {
            return Ok(agent_id.address());
        }
    }

    return Err(ZomeApiError::from(String::from(
        "AgentId entry not found in source chain",
    )));
}

pub fn get_badge_class_from_chain(
    badge_class: &Address,
    entries: &Vec<Entry>,
) -> ZomeApiResult<BadgeClass> {
    for entry in entries {
        if let Entry::App(entry_type, json) = entry {
            if entry_type.to_string() == "badge_class" && &entry.address() == badge_class {
                let class = BadgeClass::try_from(json)?;
                return Ok(class);
            }
        }
    }

    Err(ZomeApiError::from(format!(
        "Badge class {} not found in chain",
        badge_class
    )))
}
