#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]
#![forbid(unsafe_code)]

extern crate self as lets_auth;

mod all_folders;
mod config;
mod denormalized_folders;
mod first_folder;
mod folder;
pub mod schema;

pub const SYSTEM: &str = "lets-auth";
pub type AppUrl = ft_sdk::RequiredAppUrl<SYSTEM>;
pub use all_folders::all_folders;
pub use config::Config;
pub use denormalized_folders::denormalized_folders;
#[expect(unused)]
pub(crate) use folder::DbFolder;
pub use folder::{Folder, FolderID};
