use crate::badge_claim::BadgeClaim;
use crate::badge_class::BadgeClass;
use crate::utils::get_package_entries;
use hdk::prelude::*;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct BadgeAssertion {
    pub badge_class: Address,
    pub recipient: Address,
}

pub fn entry_def() -> ValidatingEntryType {
    entry!(
        name: "badge_assertion",
        description: "Badge instance of the given class for the given recipient",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },
        validation: |validation_data: hdk::EntryValidationData<BadgeAssertion>| {
            match validation_data {
                EntryValidationData::Create { entry, validation_data } => {
                    let entries = get_package_entries(&validation_data.package)?;

                    let badge_class = get_badge_class_from_chain(&entry.badge_class, &entries)?;

                    if entry.recipient == badge_class.creator_address {
                        return Ok(());
                    }

                    check_badge_claims(&entry.badge_class, &badge_class, &entry.recipient, &entries)?;

                    Ok(())
                },
                _ => Err(String::from("Cannot update or delete a badge class")),
            }
        },
        links: [
            from!(
                "%agent_id",
                link_type: "recipient->badge_assertion",
                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },
                validation: | validation_data: hdk::LinkValidationData | {
                    match validation_data {
                        hdk::LinkValidationData::LinkAdd { ..} => Ok(()),
                        _ => Err(String::from("Cannot delete links"))
                    }
                }
            )
        ]
    )
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

/**
 * Badge claims are valid if there are more actual claims than validators,
 * or if one of the claims comes from the badge creator itself
 */
pub fn check_badge_claims(
    badge_class_address: &Address,
    badge_class: &BadgeClass,
    recipient: &Address,
    entries: &Vec<Entry>,
) -> ZomeApiResult<()> {
    let mut actual_claims = 0;
    for entry in entries {
        if let Entry::App(entry_type, json) = entry {
            if entry_type.to_string() == "badge_claim" {
                let claim = BadgeClaim::try_from(json)?;

                if claim.issuer == badge_class.creator_address {
                    return Ok(());
                }
                if &claim.recipient == recipient && &claim.badge_class == badge_class_address {
                    actual_claims += 1;
                }
            }
        }
    }

    match actual_claims >= badge_class.validators {
        true => Ok(()),
        false => Err(ZomeApiError::from(format!(
            "Not enough badge claims for badge class {}, need {} but agent {} only has {}, {:?}",
            badge_class_address, badge_class.validators, recipient, actual_claims, entries
        ))),
    }
}
