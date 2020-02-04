use crate::badge_class::BadgeClass;
use hdk::prelude::*;
use hdk::AGENT_ADDRESS;

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Badge {
    pub recipient: Address,
    pub badge_class: Address,
    pub issuers: Vec<Address>,
    pub evidences: Vec<Address>,
}

impl Badge {
    /**
     * Returns the initial badge address for the given recipient and class
     */
    pub fn initial(recipient: &Address, badge_class: &Address) -> Badge {
        Badge {
            recipient: recipient.clone(),
            badge_class: badge_class.clone(),
            issuers: vec![],
            evidences: vec![],
        }
    }

    pub fn address(&self) -> ZomeApiResult<Address> {
        let entry = Entry::App("badge".into(), self.clone().into());

        hdk::entry_address(&entry)
    }
}

pub fn entry_def() -> ValidatingEntryType {
    entry!(
        name: "badge",
        description: "An instance of a badge of a class for a certain person, ",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
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

                    let author = &validation_data.sources()[0];

                    // TODO validate evidences
                    if get_new_issuer(&new_entry.issuers, &old_entry.issuers)? != author.clone() {
                        return Err(String::from("The issuer of a badge can only add themselves to the issuers list"));
                    }
                    assert_issuer_valid(&new_entry.badge_class, &author)?;
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
                        hdk::LinkValidationData::LinkAdd { link, .. } => {
                            let badge: Badge = hdk::utils::get_as_type(link.link.target().clone())?;

                            match badge.recipient == link.link.base().clone() {
                                true => Ok(()),
                                false => Err(String::from("Cannot link \"recipient->badge\" from an agent who is not the recipient of the badge"))
                            }
                        },
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
                        hdk::LinkValidationData::LinkAdd { .. } => {
                            /* let badge: Badge = hdk::utils::get_as_type(link.link.target().clone())?;

                            match badge.issuers.contains(link.link.base()) {
                                true => Ok(()),
                                false => Err(format!("Cannot link \"issuer->badge\" from an agent {} who is not the issuer of the badge {:?}", link.link.base(), badge))
                            } */
                            Ok(())
                        },
                        _ => Err(String::from("Cannot delete links"))
                    }
                }
            )
        ]
    )
}

/** Handlers */

pub fn claim_agent_deserves_badge(
    recipient: Address,
    badge_class: Address,
    evidences: Vec<Address>,
) -> ZomeApiResult<Address> {
    let mut badge = Badge::initial(&recipient, &badge_class);

    let initial_entry = Entry::App("badge".into(), badge.clone().into());

    let badge_address = hdk::entry_address(&initial_entry)?;

    let last_entry: Option<Entry> = hdk::get_entry(&badge_address)?;

    if let Some(_) = last_entry {
        badge = hdk::utils::get_as_type(badge_address.clone())?;
    } else {
        hdk::commit_entry(&initial_entry)?;
    }

    badge.issuers.push(AGENT_ADDRESS.clone());
    badge.evidences.append(&mut evidences.clone());

    let new_entry = Entry::App("badge".into(), badge.clone().into());
    let address = hdk::update_entry(new_entry, &badge_address)?;

    hdk::link_entries(
        &AGENT_ADDRESS,
        &badge_address,
        "issuer->badge",
        String::from(badge_class.clone()).as_str(),
    )?;

    let tag = match assert_issuer_valid(&badge.badge_class, &badge.recipient) {
        Ok(()) => "completed",
        Err(_) => "temptative",
    };

    hdk::link_entries(&recipient, &badge_address, "recipient->badge", tag)?;
    hdk::link_entries(&badge_class, &address, "badge_class->badge", "")?;
    Ok(badge_address)
}

/** Validation helpers */

/**
 * Badge claims are valid if there are more actual claims than validators,
 * or if one of the claims comes from the badge creator itself
 */
fn assert_issuer_valid(badge_class_address: &Address, issuer: &Address) -> ZomeApiResult<()> {
    let badge_class: BadgeClass = hdk::utils::get_as_type(badge_class_address.clone())?;

    if badge_class.creator_address == issuer.clone() {
        return Ok(());
    }

    let badge = Badge::initial(&issuer, &badge_class_address);

    let latest_badge: Badge = hdk::utils::get_as_type(badge.address()?)?;

    match latest_badge.issuers.len() >= badge_class.validators {
        true => Ok(()),
        false => Err(ZomeApiError::from(format!(
            "Issuer {} for badge {} is not valid",
            issuer, badge_class.name
        ))),
    }
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
