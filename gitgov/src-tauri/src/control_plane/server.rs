// Control Plane HTTP client and DTO facade.
//
// Keep this module as the stable public import path for Tauri commands.

mod client;
mod models;

#[cfg(test)]
mod tests;

pub use client::ControlPlaneClient;
pub use models::*;
