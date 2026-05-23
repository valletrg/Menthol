//! Profile management - persists user credentials to disk.
//!
//! Profiles are stored in ~/.config/menthol/profiles.json as a JSON array.
//! Each profile holds username + (optionally) password.
//! Passwords are stored in plaintext — TODO: consider encryption.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const PROFILE_DIR: &str = ".config/menthol";
const PROFILE_FILE: &str = "profiles.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

fn profile_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(PROFILE_DIR).join(PROFILE_FILE)
}

fn ensure_dir() -> std::io::Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = home.join(PROFILE_DIR);
    fs::create_dir_all(dir)?;
    Ok(())
}

/// Load all saved profiles. Returns empty vec if file doesn't exist.
pub fn load_profiles() -> Vec<Profile> {
    let path = profile_path();
    if !path.exists() {
        return Vec::new();
    }
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Save all profiles to disk.
pub fn save_profiles(profiles: &[Profile]) -> std::io::Result<()> {
    ensure_dir()?;
    let path = profile_path();
    let json = serde_json::to_string_pretty(profiles)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)
}

/// Add a new profile. If a profile with the same username already exists, replace it.
pub fn add_profile(profile: Profile) -> std::io::Result<()> {
    let mut profiles = load_profiles();
    // Remove existing profile with same username
    profiles.retain(|p| p.username != profile.username);
    profiles.push(profile);
    save_profiles(&profiles)
}

/// Remove a profile by username.
pub fn remove_profile(username: &str) -> std::io::Result<()> {
    let mut profiles = load_profiles();
    profiles.retain(|p| p.username != username);
    save_profiles(&profiles)
}
