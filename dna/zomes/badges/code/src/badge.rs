use crate::badge_class::BadgeClass;
use crate::utils::{get_badge_class_from_chain, get_chain_agent_id, get_package_entries};
use hdk::prelude::*;
use hdk::AGENT_ADDRESS;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Badge {
    pub recipient: Address,
    pub badge_class: Address,
    pub issuers: Vec<Address>,
    pub evidences: Vec<Address>,
}

pub fn entry_def() -> ValidatingEntryType {
    entry!(
        name: "badge",
        description: "An instance of a badge of a class for a certain person, ",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },
        validation: |validation_data: hdk::EntryValidationData<Badge>| {
            match validation_data {
                EntryValidationData::Create { entry, .. } => {
                    if entry.issuers.len() > 0 {
                        return Err(String::from("No issuers can be present when creating a Badge"));
                    }

                    if entry.evidences.len() > 0 {
                        return Err(String::from("No evidences can be present when creating a Badge"));
                    }

                    Ok(())
                },
                EntryValidationData::Modify {
                    new_entry,
                    old_entry,
                    validation_data,
                    ..
                } => {
                    if new_entry.issuers.contains(&new_entry.recipient) {
                        return Err(String::from("Badge issuers list cannot contain the badge recipient"));
                    }

                    if new_entry.recipient != old_entry.recipient {
                        return Err(String::from("Cannot change recipient of a badge"));
                    }

                    if new_entry.badge_class != old_entry.badge_class {
                        return Err(String::from("Cannot change class of a badge"));
                    }

                    let entries = get_package_entries(&validation_data.package)?;

                    let badge_class = get_badge_class_from_chain(&new_entry.badge_class, &entries)?;

                    let author = get_chain_agent_id(&entries)?;

                    // TODO validate evidences
                    if author == new_entry.recipient {
                        assert_issuers_are_equal(&old_entry.issuers, &new_entry.issuers)?;
                    } else if get_new_issuer(&new_entry.issuers, &old_entry.issuers)? == author {
                        assert_issuer_valid(new_entry.badge_class, badge_class, author, &entries)?;
                    } else {
                        return Err(String::from("Only issuers or the recipient can change "));
                    }
                    Ok(())
                },
                _ => Err(String::from("Cannot update or delete a badge class")),
            }
        },
        links: [
            from!(
                "%agent_id",
                link_type: "recipient->badge",
                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },
                validation: | validation_data: hdk::LinkValidationData | {
                    match validation_data {
                        hdk::LinkValidationData::LinkAdd{ .. } => Ok(()),
                        _ => Err(String::from("Cannot delete links"))
                    }
                }
            ),
            from!(
                "%agent_id",
                link_type: "issuer->badge",
                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },
                validation: | validation_data: hdk::LinkValidationData | {
                    match validation_data {
                        hdk::LinkValidationData::LinkAdd{ .. } => Ok(()),
                        _ => Err(String::from("Cannot delete links"))
                    }
                }
            )
        ]
    )
}

/** Exposed functions */

/**
 * Returns the initial badge address for the given recipient and class
 */
pub fn initial_badge(recipient: &Address, badge_class: &Address) -> Badge {
    Badge {
        recipient: recipient.clone(),
        badge_class: badge_class.clone(),
        issuers: vec![],
        evidences: vec![],
    }
}

/**
 * Returns the initial badge address for the given recipient and class
 */
pub fn my_badge(badge_class: &Address) -> Badge {
    Badge {
        recipient: AGENT_ADDRESS.clone(),
        badge_class: badge_class.clone(),
        issuers: vec![],
        evidences: vec![],
    }
}

pub fn badge_address(badge: Badge) -> ZomeApiResult<Address> {
    let entry = Entry::App("badge".into(), badge.into());

    hdk::entry_address(&entry)
}

pub fn update_badge_with_me_as_issuer(
    recipient: Address,
    badge_class: Address,
    evidences: Vec<Address>,
) -> ZomeApiResult<Address> {
    let mut badge = initial_badge(&recipient, &badge_class);

    let initial_entry = Entry::App("badge".into(), badge.clone().into());

    let badge_address = hdk::entry_address(&initial_entry)?;

    let last_entry: Option<Entry> = hdk::get_entry(&badge_address)?;

    if let Some(last_badge_entry) = last_entry {
        hdk::debug(format!("haaaaa {:?}", last_badge_entry.content()))?;

        badge = hdk::utils::get_as_type(badge_address.clone())?;
    } else {
        hdk::commit_entry(&initial_entry)?;
    }

    badge.issuers.push(AGENT_ADDRESS.clone());
    badge.evidences.append(&mut evidences.clone());

    let new_entry = Entry::App("badge".into(), badge.into());
    let address = hdk::update_entry(new_entry, &badge_address)?;

    hdk::link_entries(
        &AGENT_ADDRESS,
        &badge_address,
        "issuer->badge",
        String::from(badge_class.clone()).as_str(),
    )?;
    hdk::link_entries(
        &recipient,
        &badge_address,
        "recipient->badge",
        String::from(badge_class.clone()).as_str(),
    )?;
    hdk::link_entries(&badge_class, &address, "badge_class->badge", "")?;
    Ok(badge_address)
}

/** Validation helpers */

fn assert_issuers_are_equal(old_issuers: &Vec<Address>, new_issuers: &Vec<Address>) -> ZomeApiResult<()> {
    if old_issuers.len() != new_issuers.len() {
        return Err(ZomeApiError::from(format!("Recipient of a badge can't change issuers")))
    }
    
    for i in 0..old_issuers.len() {
        if old_issuers.get(i) != new_issuers.get(i) {
            return Err(ZomeApiError::from(format!("Recipient of a badge can't change issuers")))
        }
    }

    Ok(())
}

/**
 * Badge claims are valid if there are more actual claims than validators,
 * or if one of the claims comes from the badge creator itself
 */
fn assert_issuer_valid(
    badge_class_address: Address,
    badge_class: BadgeClass,
    issuer: Address,
    chain_entries: &Vec<Entry>,
) -> ZomeApiResult<()> {
    if badge_class.creator_address == issuer {
        return Ok(());
    }

    for entry in chain_entries.iter() {
        if let Entry::App(entry_type, json) = entry {
            if entry_type.to_string() == "badge" {
                let badge = Badge::try_from(json)?;
                if badge.badge_class == badge_class_address
                    && badge.recipient == issuer
                    && badge.issuers.len() >= badge_class.validators
                {
                    return Ok(());
                }
            }
        }
    }

    hdk::debug(format!(
        "hiiii {}, {:?}, {}, {:?}",
        badge_class_address, badge_class, issuer, chain_entries
    ))?;

    Err(ZomeApiError::from(format!(
        "Agent {} is not a valid issuer for badge {}",
        issuer, badge_class.name
    )))
}

fn get_new_issuer(
    new_issuers: &Vec<Address>,
    old_issuers: &Vec<Address>,
) -> ZomeApiResult<Address> {
    if new_issuers.len() != old_issuers.len() + 1 {
        return Err(ZomeApiError::from(format!(
            "Each modification can only add a new issuer"
        )));
    }

    let mut new_issuer: Option<Address> = None;

    for issuer in new_issuers {
        if !old_issuers.contains(issuer) {
            if let Some(_) = new_issuer {
                return Err(ZomeApiError::from(format!(
                    "There is more than one new issuer for an update for a badge"
                )));
            }
            new_issuer = Some(issuer.clone());
        }
    }

    match new_issuer {
        None => Err(ZomeApiError::from(format!(
            "No new issuer found updating a badge"
        ))),
        Some(issuer) => Ok(issuer),
    }
}
