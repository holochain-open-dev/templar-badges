use hdk::error::ZomeApiError;
use hdk::prelude::*;

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
