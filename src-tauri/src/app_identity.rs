//! Which app identity this build runs under.
//!
//! Release builds use `com.dictus.desktop`. Dev and debug builds are merged
//! with `tauri.dev.conf.json` (see `scripts/tauri.ts`) and run as
//! `com.dictus.desktop.dev` / "Dictus Dev", so they get their own data folder,
//! logs, single-instance lock, autostart entry and macOS permissions, and never
//! touch the installed app's data.
//!
//! Everything that depends on the identity must derive it from
//! `app.config().identifier` / `app.package_info().name`, never a literal.

use std::path::PathBuf;

/// Suffix that marks the dev identity (`com.dictus.desktop.dev`).
const DEV_IDENTIFIER_SUFFIX: &str = ".dev";

/// `true` when `identifier` is the dev identity. The updater is disabled there:
/// a dev build must never offer to replace itself with a release.
pub fn is_dev_identity(identifier: &str) -> bool {
    identifier.ends_with(DEV_IDENTIFIER_SUFFIX)
}

/// Unix socket used by `tauri-plugin-single-instance` on macOS for this
/// identifier. Mirrors the plugin's own derivation (2.x, without the `semver`
/// feature): dots and dashes become underscores, under `/tmp`.
pub fn single_instance_socket_path(identifier: &str) -> PathBuf {
    let id = identifier.replace(['.', '-'], "_");
    PathBuf::from(format!("/tmp/{id}_si.sock"))
}

/// Remove the single-instance socket left behind by a crashed instance of
/// *this* identity. A live socket (another instance of the same identity
/// answers) is kept so the plugin can hand off to it.
#[cfg(unix)]
pub fn remove_stale_single_instance_socket(identifier: &str) {
    let socket = single_instance_socket_path(identifier);
    if socket.exists() && std::os::unix::net::UnixStream::connect(&socket).is_err() {
        eprintln!(
            "Removing stale single-instance socket: {}",
            socket.display()
        );
        let _ = std::fs::remove_file(&socket);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RELEASE: &str = "com.dictus.desktop";
    const DEV: &str = "com.dictus.desktop.dev";

    #[test]
    fn release_identity_is_not_dev() {
        assert!(!is_dev_identity(RELEASE));
    }

    #[test]
    fn dev_identity_is_dev() {
        assert!(is_dev_identity(DEV));
    }

    #[test]
    fn release_socket_path_is_unchanged() {
        assert_eq!(
            single_instance_socket_path(RELEASE),
            PathBuf::from("/tmp/com_dictus_desktop_si.sock")
        );
    }

    #[test]
    fn dev_socket_path_differs_from_release() {
        assert_eq!(
            single_instance_socket_path(DEV),
            PathBuf::from("/tmp/com_dictus_desktop_dev_si.sock")
        );
        assert_ne!(
            single_instance_socket_path(DEV),
            single_instance_socket_path(RELEASE)
        );
    }

    #[test]
    fn socket_path_replaces_dashes() {
        assert_eq!(
            single_instance_socket_path("com.dictus-desktop.dev"),
            PathBuf::from("/tmp/com_dictus_desktop_dev_si.sock")
        );
    }

    #[test]
    fn dev_config_sets_dev_identity() {
        let dev: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.dev.conf.json")).unwrap();
        assert_eq!(dev["identifier"], DEV);
        assert_eq!(dev["productName"], "Dictus Dev");
        assert!(is_dev_identity(dev["identifier"].as_str().unwrap()));
    }

    #[test]
    fn release_config_keeps_release_identity() {
        let release: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(release["identifier"], RELEASE);
        assert_eq!(release["productName"], "Dictus");
        assert!(!is_dev_identity(release["identifier"].as_str().unwrap()));
    }
}
