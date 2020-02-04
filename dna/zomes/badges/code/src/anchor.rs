use hdk::prelude::*;

pub fn entry_def() -> ValidatingEntryType {
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
                        hdk::LinkValidationData::LinkAdd { .. } => Ok(()),
                        _ => Err(String::from("Cannot delete links"))
                    }
                }
            )
        ]
    )
}

pub fn address() -> ZomeApiResult<Address> {
    let entry = Entry::App("anchor".into(), "all_badges_classes".into());

    let anchor_address = hdk::entry_address(&entry)?;

    if let None = hdk::get_entry(&anchor_address)? {
        hdk::commit_entry(&entry)?;
    }

    Ok(anchor_address)
}
