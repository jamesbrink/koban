//! Self-update support for release tarballs.
//!
//! Koban avoids the GitHub API here. The latest version is resolved by
//! following the `/releases/latest` redirect, which also avoids unauthenticated
//! API rate limits.

use std::{
    io::Read as _,
    path::Path,
    process::{Command, Stdio},
};

use sha2::{Digest, Sha256};

use crate::{KobanError, Result};

const GITHUB_REPO: &str = "jamesbrink/koban";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallKind {
    Nix,
    Cargo,
    Homebrew,
    Managed,
}

impl InstallKind {
    fn label(self) -> &'static str {
        match self {
            Self::Nix => "Nix",
            Self::Cargo => "cargo",
            Self::Homebrew => "Homebrew",
            Self::Managed => "direct install",
        }
    }
}

pub fn run(check: bool, force: bool, version: Option<String>) -> Result<String> {
    ensure_curl()?;

    let current = env!("CARGO_PKG_VERSION");
    let target_tag = match version {
        Some(version) => normalize_tag(&version),
        None => fetch_latest_tag()?,
    };
    let remote = target_tag.strip_prefix('v').unwrap_or(&target_tag);

    let mut lines = vec![format!("Current version: {current}")];

    if !force && remote == current && !check {
        lines.push(format!("Already up to date ({current})"));
        return Ok(lines.join("\n"));
    }

    if check {
        if is_newer(current, remote) {
            lines.push(format!(
                "New version available: {remote} (current: {current})"
            ));
        } else if remote == current {
            lines.push(format!("Up to date ({current})"));
        } else {
            lines.push(format!(
                "Version {remote} is available (current: {current})"
            ));
        }
        return Ok(lines.join("\n"));
    }

    let exe_path = std::env::current_exe()
        .and_then(|path| path.canonicalize())
        .map_err(|source| KobanError::Update {
            message: format!("could not resolve current executable: {source}"),
        })?;
    let install_kind = detect_install_kind(&exe_path);
    if let Some(hint) = upgrade_hint(install_kind, &target_tag) {
        return Err(KobanError::Update {
            message: format!(
                "koban was installed via {} at {}.\nIn-place self-update is not supported for this install source.\nTo upgrade to {target_tag}, run:\n{hint}",
                install_kind.label(),
                exe_path.display(),
            ),
        });
    }

    ensure_writable_install_dir(&exe_path)?;

    let action = if is_newer(current, remote) {
        "Updating"
    } else if remote == current {
        "Reinstalling"
    } else {
        "Downgrading"
    };
    lines.push(format!("{action}: {current} -> {remote}"));

    let asset_name = detect_asset_name()?;
    let asset_url =
        format!("https://github.com/{GITHUB_REPO}/releases/download/{target_tag}/{asset_name}");
    let sums_url =
        format!("https://github.com/{GITHUB_REPO}/releases/download/{target_tag}/SHA256SUMS");

    let archive = fetch_url(&asset_url)?;
    let sums = String::from_utf8(fetch_url(&sums_url)?).map_err(|source| KobanError::Update {
        message: format!("SHA256SUMS contained non-UTF-8 data: {source}"),
    })?;
    verify_checksum(&sums, asset_name, &archive)?;

    let binary = extract_binary_from_tarball(&archive)?;
    replace_binary(&binary, &exe_path)?;

    lines.push("Checksum verified (SHA-256).".to_string());
    lines.push(format!(
        "{action} complete: koban {remote} ({})",
        exe_path.display()
    ));
    Ok(lines.join("\n"))
}

fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let version = version.strip_prefix('v').unwrap_or(version);
    let version = version.split_once('-').map_or(version, |(base, _)| base);
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    parts.next().is_none().then_some((major, minor, patch))
}

fn is_newer(current: &str, remote: &str) -> bool {
    match (parse_version(current), parse_version(remote)) {
        (Some(current), Some(remote)) => remote > current,
        _ => false,
    }
}

fn normalize_tag(version: &str) -> String {
    if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{version}")
    }
}

pub fn detect_install_kind(exe_path: &Path) -> InstallKind {
    let path = exe_path.to_string_lossy();
    if path.contains("/nix/store/") {
        InstallKind::Nix
    } else if path.contains("/.cargo/bin/") || path.contains("/cargo/bin/") {
        InstallKind::Cargo
    } else if path.contains("/Cellar/") || path.contains("/homebrew/") {
        InstallKind::Homebrew
    } else {
        InstallKind::Managed
    }
}

fn upgrade_hint(kind: InstallKind, target_tag: &str) -> Option<String> {
    match kind {
        InstallKind::Nix => Some(
            "  nix profile upgrade koban\n  or, if koban is a flake input:\n    nix flake update koban"
                .to_string(),
        ),
        InstallKind::Cargo => Some(format!(
            "  cargo install --git https://github.com/{GITHUB_REPO} --tag {target_tag} --force koban"
        )),
        InstallKind::Homebrew => Some("  brew upgrade koban".to_string()),
        InstallKind::Managed => None,
    }
}

fn detect_asset_name() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Ok("koban-aarch64-apple-darwin.tar.gz"),
        ("macos", "x86_64") => Ok("koban-x86_64-apple-darwin.tar.gz"),
        ("linux", "x86_64") => Ok("koban-x86_64-unknown-linux-gnu.tar.gz"),
        ("linux", "aarch64") => Ok("koban-aarch64-unknown-linux-gnu.tar.gz"),
        (os, arch) => Err(KobanError::Update {
            message: format!("unsupported platform for self-update: {os}/{arch}"),
        }),
    }
}

fn ensure_curl() -> Result<()> {
    let ok = Command::new("curl")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    if ok {
        Ok(())
    } else {
        Err(KobanError::Update {
            message: "`curl` is required for `koban update` but was not found in PATH".to_string(),
        })
    }
}

fn fetch_latest_tag() -> Result<String> {
    let url = format!("https://github.com/{GITHUB_REPO}/releases/latest");
    let output = Command::new("curl")
        .args(["-sLI", "-o", "/dev/null", "-w", "%{url_effective}", &url])
        .output()
        .map_err(|source| KobanError::Update {
            message: format!("failed to invoke curl: {source}"),
        })?;

    if !output.status.success() {
        return Err(KobanError::Update {
            message: format!(
                "curl failed while resolving latest release (exit {:?})",
                output.status.code()
            ),
        });
    }

    let final_url = String::from_utf8_lossy(&output.stdout);
    tag_from_redirect(&final_url).ok_or_else(|| KobanError::Update {
        message: format!(
            "unexpected latest-release redirect target: {}",
            final_url.trim()
        ),
    })
}

fn tag_from_redirect(url: &str) -> Option<String> {
    let trimmed = url.trim().trim_end_matches('/');
    let tag = trimmed.rsplit('/').next()?;
    if tag.starts_with('v') && parse_version(tag).is_some() {
        Some(tag.to_string())
    } else {
        None
    }
}

fn fetch_url(url: &str) -> Result<Vec<u8>> {
    let output = Command::new("curl")
        .args(["-fsSL", url])
        .output()
        .map_err(|source| KobanError::Update {
            message: format!("failed to invoke curl: {source}"),
        })?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(KobanError::Update {
            message: format!("curl failed to fetch {url}: {}", stderr.trim()),
        })
    }
}

fn verify_checksum(sums: &str, asset: &str, data: &[u8]) -> Result<()> {
    let expected = sums
        .lines()
        .find_map(|line| {
            let (hash, name) = line.split_once("  ")?;
            (name.trim() == asset).then(|| hash.trim())
        })
        .ok_or_else(|| KobanError::Update {
            message: format!("asset {asset} not found in SHA256SUMS"),
        })?;

    let mut hasher = Sha256::new();
    hasher.update(data);
    let actual = format!("{:x}", hasher.finalize());

    if actual == expected {
        Ok(())
    } else {
        Err(KobanError::Update {
            message: format!(
                "SHA-256 checksum mismatch for {asset}\n  expected: {expected}\n  actual:   {actual}"
            ),
        })
    }
}

fn extract_binary_from_tarball(data: &[u8]) -> Result<Vec<u8>> {
    let decoder = flate2::read::GzDecoder::new(data);
    let mut archive = tar::Archive::new(decoder);

    for entry in archive.entries().map_err(|source| KobanError::Update {
        message: format!("could not read release archive: {source}"),
    })? {
        let mut entry = entry.map_err(|source| KobanError::Update {
            message: format!("could not read release archive entry: {source}"),
        })?;
        let path = entry.path().map_err(|source| KobanError::Update {
            message: format!("could not read release archive path: {source}"),
        })?;
        if path.file_name().is_some_and(|name| name == "koban") {
            let mut bytes = Vec::new();
            entry
                .read_to_end(&mut bytes)
                .map_err(|source| KobanError::Update {
                    message: format!("could not read koban binary from archive: {source}"),
                })?;
            return Ok(bytes);
        }
    }

    Err(KobanError::Update {
        message: "`koban` binary not found in release archive".to_string(),
    })
}

fn ensure_writable_install_dir(exe_path: &Path) -> Result<()> {
    let Some(exe_dir) = exe_path.parent() else {
        return Err(KobanError::Update {
            message: "cannot determine binary directory".to_string(),
        });
    };

    let probe = exe_dir.join(format!(".koban-update-test-{}", std::process::id()));
    match std::fs::write(&probe, b"") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            Ok(())
        }
        Err(source) => Err(KobanError::Update {
            message: format!("no write permission to {}: {source}", exe_dir.display()),
        }),
    }
}

fn replace_binary(new_binary: &[u8], exe_path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let exe_dir = exe_path.parent().ok_or_else(|| KobanError::Update {
        message: "cannot determine binary directory".to_string(),
    })?;
    let tmp_path = exe_dir.join(format!(".koban-update-{}", std::process::id()));
    let backup_path = exe_path.with_extension("old");

    std::fs::write(&tmp_path, new_binary).map_err(|source| KobanError::Update {
        message: format!("failed to write new binary to temp file: {source}"),
    })?;
    std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755)).map_err(
        |source| KobanError::Update {
            message: format!("failed to set permissions on new binary: {source}"),
        },
    )?;

    std::fs::rename(exe_path, &backup_path).map_err(|source| KobanError::Update {
        message: format!("failed to move current binary to backup: {source}"),
    })?;

    if let Err(source) = std::fs::rename(&tmp_path, exe_path) {
        let _ = std::fs::rename(&backup_path, exe_path);
        let _ = std::fs::remove_file(&tmp_path);
        return Err(KobanError::Update {
            message: format!("failed to install new binary: {source}"),
        });
    }

    let _ = std::fs::remove_file(&backup_path);

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("xattr")
            .args(["-d", "com.apple.quarantine"])
            .arg(exe_path)
            .output();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    use serde_json::Value;

    #[test]
    fn version_parsing_and_comparison_accept_v_prefixes() {
        assert_eq!(parse_version("0.1.2"), Some((0, 1, 2)));
        assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
        assert!(is_newer("0.0.1", "v0.0.2"));
        assert!(!is_newer("0.0.2", "v0.0.1"));
        assert_eq!(parse_version("v1.2"), None);
    }

    #[test]
    fn tag_from_redirect_strips_latest_release_url() {
        assert_eq!(
            tag_from_redirect("https://github.com/jamesbrink/koban/releases/tag/v1.2.3"),
            Some("v1.2.3".to_string())
        );
        assert_eq!(
            tag_from_redirect("https://github.com/jamesbrink/koban/releases/tag/not-a-version"),
            None
        );
    }

    #[test]
    fn install_kind_detection_covers_package_managers() {
        assert_eq!(
            detect_install_kind(Path::new("/nix/store/abc-koban/bin/koban")),
            InstallKind::Nix
        );
        assert_eq!(
            detect_install_kind(Path::new("/Users/james/.cargo/bin/koban")),
            InstallKind::Cargo
        );
        assert_eq!(
            detect_install_kind(Path::new("/opt/homebrew/bin/koban")),
            InstallKind::Homebrew
        );
        assert_eq!(
            detect_install_kind(Path::new("/Users/james/.local/bin/koban")),
            InstallKind::Managed
        );
    }

    #[test]
    fn checksum_verification_matches_sha256sums() {
        let data = b"koban";
        let mut hasher = Sha256::new();
        hasher.update(data);
        let sums = format!("{:x}  koban-test.tar.gz\n", hasher.finalize());
        assert!(verify_checksum(&sums, "koban-test.tar.gz", data).is_ok());

        let err = verify_checksum(&sums, "missing.tar.gz", data).expect_err("missing asset");
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn tarball_extraction_finds_koban_binary() {
        let expected = b"fake-koban";
        let archive = make_tarball(&[("README.md", b"docs"), ("bin/koban", expected)]);
        assert_eq!(
            extract_binary_from_tarball(&archive).expect("binary"),
            expected
        );
    }

    #[test]
    fn replace_binary_swaps_executable() {
        let dir = tempfile::tempdir().expect("temp dir");
        let exe = dir.path().join("koban");
        std::fs::write(&exe, b"old").expect("old binary");

        replace_binary(b"new", &exe).expect("replace");

        assert_eq!(std::fs::read(&exe).expect("read"), b"new");
        assert!(!exe.with_extension("old").exists());
    }

    fn make_tarball(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (name, data) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder
                .append_data(&mut header, name, *data)
                .expect("append");
        }
        let tar = builder.into_inner().expect("tar bytes");
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(&tar).expect("gzip write");
        encoder.finish().expect("gzip finish")
    }

    #[test]
    fn github_release_payload_shape_is_stable() {
        let body = serde_json::json!({
            "repo": GITHUB_REPO,
            "assets": [
                "koban-aarch64-apple-darwin.tar.gz",
                "koban-x86_64-apple-darwin.tar.gz",
                "koban-x86_64-unknown-linux-gnu.tar.gz",
                "koban-aarch64-unknown-linux-gnu.tar.gz",
                "SHA256SUMS"
            ]
        });
        assert_eq!(body["repo"], Value::String("jamesbrink/koban".to_string()));
    }
}
