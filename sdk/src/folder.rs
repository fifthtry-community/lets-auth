#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct FolderID(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Folder {
    pub guid: FolderID,
    pub kind: Option<String>,
    pub parents: Vec<FolderID>,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = lets_auth::schema::fastn_folder)]
#[diesel(check_for_backend(ft_sdk::Sqlite))]
pub(crate) struct DbFolder {
    guid: String,
    kind: Option<String>,
    parents: String,

    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,

}

impl DbFolder {
    #[expect(unused)]
    pub(crate) fn into_folder(self) -> ft_sdk::Result<Folder> {
        Ok(Folder {
            guid: FolderID(self.guid),
            kind: self.kind,
            parents: serde_json::from_str(&self.parents)?,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }}
