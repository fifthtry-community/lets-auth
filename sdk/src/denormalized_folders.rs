pub fn denormalized_folders(
    conn: &mut ft_sdk::Connection,
    folders: Vec<lets_auth::FolderID>,
) -> ft_sdk::Result<std::collections::HashSet<lets_auth::FolderID>> {
    use diesel::prelude::*;
    use lets_auth::schema::fastn_folder;

    let mut all_folders = std::collections::HashSet::new();
    let mut stack: Vec<String> = folders.into_iter().map(|f| f.0).collect();

    while !stack.is_empty() {
        let results: Vec<(String, String)> = fastn_folder::table
            .filter(fastn_folder::guid.eq_any(&stack))
            .select((fastn_folder::guid, fastn_folder::parents))
            .load(conn)?;

        stack.clear();

        for (guid, parents) in results {
            let folder_id = lets_auth::FolderID(guid);
            if all_folders.insert(folder_id) {
                let parent_ids: Vec<String> = serde_json::from_str(&parents)?;
                stack.extend(parent_ids);
            }
        }
    }

    Ok(all_folders)
}
