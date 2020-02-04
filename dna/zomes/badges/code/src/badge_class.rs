use hdk::prelude::*;

use crate::badge::Badge;

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct BadgeClass {
    pub name: String,
    pub description: String,
    pub creator_address: Address,
    pub image: String,
    pub validators: usize,
}

pub fn entry_def() -> ValidatingEntryType {
    entry!(
        name: "badge_class",
        description: "Information about a type of badge",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |validation_data: hdk::EntryValidationData<BadgeClass>| {
            match validation_data {
                EntryValidationData::Create { .. } => {
                /*  let creator_address = entry.creator_address;

                    if !validation_data.clone().sources().contains(&creator_address) {
                        return Err(format!(
                            "The creator {} of badge class {} must sign its creation, sources {:?}",
                            creator_address,
                            validation_data.package.chain_header.entry_address(),
                            validation_data.sources()
                        ));
                    }
                */
                   Ok(())
                },
                _ => Err(String::from("Cannot update or delete a badge class")),
            }
        },
        links: [
            to!(
                "badge",
                link_type: "badge_class->badge",
                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },
                validation: | validation_data: hdk::LinkValidationData | {
                    match validation_data {
                        hdk::LinkValidationData::LinkAdd { link, .. } => {
                            let badge: Badge = hdk::utils::get_as_type(link.link.target().clone())?;

                            match badge.badge_class == link.link.base().clone() {
                                true => Ok(()),
                                false => Err(String::from("Cannot link \"badge_class->badge\" to a badge that is not the class of the base class"))
                            }
                        },
                        _ => Err(String::from("Cannot delete links"))
                    }
                }
            ),
            from!(
                "%agent_id",
                link_type: "creator->badge_class",
                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },
                validation: | validation_data: hdk::LinkValidationData | {
                    match validation_data {
                        hdk::LinkValidationData::LinkAdd { link, .. } => {
                            let badge_class: BadgeClass = hdk::utils::get_as_type(link.link.target().clone())?;

                            match badge_class.creator_address == link.link.base().clone() {
                                true => Ok(()),
                                false => Err(String::from("Cannot link \"creator->badge_class\" to a badge class that was not created by the base address"))
                            }
                        },
                        _ => Err(String::from("Cannot delete links"))
                    }
                }
            )
        ]
    )
}
