pub mod net;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use crate::linux as platform;

#[cfg(target_family = "windows")]
pub mod windows;
#[cfg(target_family = "windows")]
pub use crate::windows as platform;