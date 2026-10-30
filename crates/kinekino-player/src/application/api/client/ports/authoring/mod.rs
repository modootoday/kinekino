//! Port traits of the parent module (hexagonal `ports` location): authoring projects.
use super::*;

mod assets;
mod cards;
mod clips;
mod projects;
pub use {assets::AssetClient, cards::CardClient, clips::ClipClient, projects::ProjectClient};
