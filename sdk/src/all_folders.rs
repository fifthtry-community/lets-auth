#[derive(Debug)]
pub struct Folders {
    pub folders: Vec<lets_auth::FolderID>,
    pub denormalized_folders: Vec<lets_auth::FolderID>,
}

pub fn all_folders(conn: &mut ft_sdk::Connection, uid: i64) -> ft_sdk::Result<Folders> {
    use diesel::prelude::*;
    use lets_auth::schema::fastn_user;

    let (denormalized_folders, folders) = fastn_user::table
        .filter(fastn_user::id.eq(uid))
        .select((fastn_user::denormalized_folders, fastn_user::folders))
        .get_result::<(String, String)>(conn)?;

    // serde handles "new type" structs like FolderID "correctly" so we can just deserialize.
    let denormalized_folders = serde_json::from_str(&denormalized_folders)?;
    let folders = serde_json::from_str(&folders)?;

    Ok(Folders {
        folders,
        denormalized_folders,
    })
}
