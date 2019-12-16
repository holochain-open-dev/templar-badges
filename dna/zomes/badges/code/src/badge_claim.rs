use crate::badge_assertion::BadgeAssertion;
use crate::utils::{get_chain_agent_id, get_package_entries};
use hdk::prelude::*;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct BadgeClaim {
    pub issuer: Address,
    pub recipient: Address,
    pub badge_class: Address,
    pub evidences: Vec<Address>,
}

pub fn entry_def() -> ValidatingEntryType {
    entry!(
        name: "badge_claim",
        description: "The issuer claims that the recipient should have the specified badge class because of the evidences they present",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },
        validation: |validation_data: hdk::EntryValidationData<BadgeClaim>| {
            match validation_data {
                EntryValidationData::Create {  validation_data, entry } => {
                    if entry.recipient == entry.issuer {
                        return Err(String::from("Claim recipient and issuer must be different"));
                    }

                    let entries = get_package_entries(&validation_data.package)?;

                    let author = get_chain_agent_id(&entries)?;
                    let sources = validation_data.sources();

                    // If we are validating the recipient's chain, check that the issuer had already committed before that
                    if author == entry.recipient {
                        if !sources.contains(&entry.recipient) {
                            return Err(String::from("The badge claim should be signed by the recipient"));
                        }
                        if !sources.contains(&entry.issuer) {
                            return Err(String::from("The badge claim should be signed by the issuer"));
                        }
                    } else if author == entry.issuer {
                        check_badge_assertion_in_chain(entry.badge_class, entry.issuer, &entries)?;
                    } else {
                        return Err(String::from("Only issuer or recipient can commit a BadgeClaim"));
                    }

                    Ok(())
                },
                _ => Err(String::from("Cannot update or delete a badge class")),
            }
        },
        links: [
            from!(
                "%agent_id",
                link_type: "recipient->badge_claim",
                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },
                validation: | validation_data: hdk::LinkValidationData | {
                    match validation_data {
                        hdk::LinkValidationData::LinkAdd{ ..} => Ok(()),
                        _ => Err(String::from("Cannot delete links"))
                    }
                }
            ),
            from!(
                "%agent_id",
                link_type: "issuer->badge_claim",
                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },
                validation: | validation_data: hdk::LinkValidationData | {
                    match validation_data {
                        hdk::LinkValidationData::LinkAdd{ ..} => Ok(()),
                        _ => Err(String::from("Cannot delete links"))
                    }
                }
            )
        ]
    )
}

pub fn check_badge_assertion_in_chain(
    badge_class: Address,
    recipient: Address,
    chain_entries: &Vec<Entry>,
) -> ZomeApiResult<()> {
    for entry in chain_entries.iter() {
        if let Entry::App(entry_type, json) = entry {
            if entry_type.to_string() == "badge_assertion" {
                let assertion: BadgeAssertion = BadgeAssertion::try_from(json)?;
                if assertion.badge_class == badge_class && assertion.recipient == recipient {
                    return Ok(());
                }
            }
        }
    }

    Err(ZomeApiError::from(format!(
        "Badge assertion for badge class {} not found in chain",
        badge_class
    )))
}
