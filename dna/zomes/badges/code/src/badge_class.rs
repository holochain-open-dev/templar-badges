use hdk::prelude::*;

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct BadgeClass {
    pub name: String,
    pub description: String,
    pub creator_address: Address,
    pub image: String,
    pub validators: usize,
}

pub fn anchor_entry_def() -> ValidatingEntryType {
    entry!(
        name: "anchor",
        description: "Anchor to all badge classes",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |validation_data: hdk::EntryValidationData<String>| {
            match validation_data {
                EntryValidationData::Create { .. } => {
                    Ok(())
                },
                _ => Err(String::from("Cannot update or delete an anchor")),
            }
        },
        links: [
            to!(
                "badge_class",
                link_type: "anchor->badge_class",
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
                EntryValidationData::Create { entry, validation_data } => {
                    let creator_address = entry.creator_address;

                    if !validation_data.clone().sources().contains(&creator_address) {
                        return Err(format!(
                            "The creator {} of badge class {} must sign its creation, sources {:?}",
                            creator_address,
                            validation_data.package.chain_header.entry_address(),
                            validation_data.sources()
                        ));
                    }

                   Ok(())
                },
                _ => Err(String::from("Cannot update or delete a badge class")),
            }
        },
        links: [
            to!(
                "badge_claim",
                link_type: "badge_class->badge_claim",
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
            to!(
                "badge_assertion",
                link_type: "badge_class->badge_assertion",
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
