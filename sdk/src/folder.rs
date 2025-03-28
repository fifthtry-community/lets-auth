#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
/// FolderID in uuidv4 format
pub struct FolderID(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Folder {
    pub guid: FolderID,
    pub name: String,
    pub is_exception: bool,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = lets_auth::schema::fastn_folder)]
#[diesel(check_for_backend(ft_sdk::Sqlite))]
pub(crate) struct DbFolder {
    guid: String,
    name: String,
    is_exception: bool,

    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl DbFolder {
    pub(crate) fn into_folder(self) -> Folder {
        Folder {
            guid: FolderID(self.guid),
            name: self.name,
            is_exception: self.is_exception,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// FolderInput is a way to specify a folder by either its ID or its path.
pub enum FolderInput {
    Id(FolderID),
    Path(FolderPath),
}

/// FolderPath is a path to a folder in the form of "/folder-name/"
pub struct FolderPath(pub String);

impl FolderPath {
    pub fn new(path: impl AsRef<str>) -> Self {
        assert!(!path.as_ref().is_empty());
        assert!(path.as_ref().starts_with('/'), "path must start with a /");
        assert!(path.as_ref().ends_with('/'), "path must end with a /");

        Self(path.as_ref().to_string())
    }
}

/// Check if the user has a permission on a folder
/// The user must belong to the folder or one of its parents[1] to have permission.
///
/// Errors:
/// - `folder not found` if the folder does not exist
/// - `multiple folders found: N` if the folder is not unique
///
/// [1]: Parents (nested folders) are not supported yet. Path must be like /folder-name/.
pub fn has_perms_on_folder(
    conn: &mut ft_sdk::Connection,
    uid: ft_sdk::UserId,
    permission: impl AsRef<str>,
    folder: FolderInput,
) -> ft_sdk::Result<bool> {
    use crate::schema::{fastn_folder_permission, fastn_folder_user};
    use diesel::prelude::*;

    let folder = get_folder_from_folder_input(conn, folder)?;

    // check if the user is associated to that folder
    let user_is_associated_to_folder = diesel::select(diesel::dsl::exists(
        fastn_folder_user::table
            .filter(fastn_folder_user::uid.eq(uid.0))
            .filter(fastn_folder_user::fguid.eq(&folder.guid.0)),
    ))
    .get_result::<bool>(conn)?;

    if !user_is_associated_to_folder {
        return Ok(false);
    }

    // check if permission is applied to the user. use fastn_folder_permission
    let permission_is_applied = diesel::select(diesel::dsl::exists(
        fastn_folder_permission::table
            .filter(fastn_folder_permission::fguid.eq(&folder.guid.0))
            .filter(fastn_folder_permission::permission.eq(permission.as_ref())),
    ))
    .get_result::<bool>(conn)?;

    Ok(permission_is_applied)
}

fn get_folder_from_folder_input(
    conn: &mut ft_sdk::Connection,
    finput: FolderInput,
) -> ft_sdk::Result<Folder> {
    use crate::schema::fastn_folder;
    use diesel::prelude::*;

    let folder = match finput {
        FolderInput::Id(fguid) => {
            let folder: DbFolder = match fastn_folder::table
                .filter(fastn_folder::guid.eq(&fguid.0))
                .select(DbFolder::as_select())
                .load::<DbFolder>(conn)?
            {
                v => {
                    if v.len() == 0 {
                        return Err(ft_sdk::anyhow!("folder not found"));
                    }
                    if v.len() > 1 {
                        return Err(ft_sdk::anyhow!("multiple folders found: {}", v.len()));
                    }
                    v.into_iter().next().unwrap()
                }
            };

            folder.into_folder()
        }
        FolderInput::Path(fpath) => {
            // TODO(siddhantk232): support nested folders
            if fpath.0.chars().filter(|c| *c == '/').count() > 2 {
                return Err(ft_sdk::anyhow!("nested folders are not supported yet"));
            }

            let folder_name = fpath.0.trim_matches('/');

            let folder: DbFolder = match fastn_folder::table
                .filter(fastn_folder::name.eq(folder_name))
                .select(DbFolder::as_select())
                .load::<DbFolder>(conn)?
            {
                v => {
                    if v.len() == 0 {
                        return Err(ft_sdk::anyhow!("folder not found"));
                    }
                    if v.len() > 1 {
                        return Err(ft_sdk::anyhow!("multiple folders found: {}", v.len()));
                    }
                    v.into_iter().next().unwrap()
                }
            };

            folder.into_folder()
        }
    };

    Ok(folder)
}
