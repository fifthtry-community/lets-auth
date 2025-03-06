#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]
#![forbid(unsafe_code)]

extern crate self as lets_auth;

mod config;
mod first_folder;
mod all_folders;
pub mod schema;
mod folder;

pub const SYSTEM: &str = "lets-auth";
pub type AppUrl = ft_sdk::RequiredAppUrl<SYSTEM>;
pub use config::Config;
pub use folder::{Folder, FolderID};
#[expect(unused)]
pub(crate) use folder::DbFolder;
pub use all_folders::all_folders;
