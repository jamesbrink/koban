//! Self-update support for release tarballs.
//!
//! Koban avoids the GitHub API here for stable updates. The latest version is
//! resolved by following the `/releases/latest` redirect, which also avoids
//! unauthenticated API rate limits. Nightlies are fetched from the rolling
//! `nightly` prerelease.

use std::{
    io::Read as _,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use sha2::{Digest, Sha256};

use koban::{KobanError, Result};

const GITHUB_REPO: &str = "jamesbrink/koban";
const NIGHTLY_TAG: &str = "nightly";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstallKind {
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

pub fn run(check: bool, force: bool, version: Option<String>, nightly: bool) -> Result<String> {
    run_with_ops(&SystemOps, check, force, version, nightly)
}

trait UpdateOps {
    fn ensure_curl(&self) -> Result<()>;
    fn fetch_latest_tag(&self) -> Result<String>;
    fn current_exe(&self) -> Result<PathBuf>;
    fn fetch_url(&self, url: &str) -> Result<Vec<u8>>;
    fn replace_binary(&self, new_binary: &[u8], exe_path: &Path) -> Result<()>;
}

struct SystemOps;

impl UpdateOps for SystemOps {
    fn ensure_curl(&self) -> Result<()> {
        ensure_curl()
    }

    fn fetch_latest_tag(&self) -> Result<String> {
        fetch_latest_tag()
    }

    fn current_exe(&self) -> Result<PathBuf> {
        std::env::current_exe()
            .and_then(|path| path.canonicalize())
            .map_err(|source| KobanError::Update {
                message: format!("could not resolve current executable: {source}"),
            })
    }

    fn fetch_url(&self, url: &str) -> Result<Vec<u8>> {
        fetch_url(url)
    }

    fn replace_binary(&self, new_binary: &[u8], exe_path: &Path) -> Result<()> {
        replace_binary(new_binary, exe_path)
    }
}

fn run_with_ops(
    ops: &impl UpdateOps,
    check: bool,
    force: bool,
    version: Option<String>,
    nightly: bool,
) -> Result<String> {
    let current = env!("CARGO_PKG_VERSION");
    let target_tag = if nightly {
        NIGHTLY_TAG.to_string()
    } else {
        match version {
            Some(version) => normalize_tag(&version),
            None => {
                ops.ensure_curl()?;
                ops.fetch_latest_tag()?
            }
        }
    };
    validate_tag(&target_tag)?;
    let remote = target_tag.strip_prefix('v').unwrap_or(&target_tag);

    let mut lines = vec![format!("Current version: {current}")];

    if !nightly && !force && remote == current && !check {
        lines.push(format!("Already up to date ({current})"));
        return Ok(lines.join("\n"));
    }

    if check {
        if nightly {
            lines.push("Nightly build available from the nightly release.".to_string());
        } else if is_newer(current, remote) {
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

    if !nightly && !force && !is_newer(current, remote) {
        return Err(KobanError::Update {
            message: format!("refusing to downgrade from {current} to {remote} without --force"),
        });
    }

    let exe_path = ops.current_exe()?;
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

    let action = if nightly {
        "Installing nightly"
    } else if is_newer(current, remote) {
        "Updating"
    } else if remote == current {
        "Reinstalling"
    } else {
        "Downgrading"
    };
    lines.push(format!("{action}: {current} -> {remote}"));

    ops.ensure_curl()?;

    let asset_name = detect_asset_name()?;
    let asset_url =
        format!("https://github.com/{GITHUB_REPO}/releases/download/{target_tag}/{asset_name}");
    let sums_url =
        format!("https://github.com/{GITHUB_REPO}/releases/download/{target_tag}/SHA256SUMS");

    let archive = ops.fetch_url(&asset_url)?;
    let sums =
        String::from_utf8(ops.fetch_url(&sums_url)?).map_err(|source| KobanError::Update {
            message: format!("SHA256SUMS contained non-UTF-8 data: {source}"),
        })?;
    verify_checksum(&sums, asset_name, &archive)?;

    let binary = extract_binary_from_tarball(&archive)?;
    ops.replace_binary(&binary, &exe_path)?;

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
    if version == NIGHTLY_TAG || version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{version}")
    }
}

fn validate_tag(tag: &str) -> Result<()> {
    if tag == NIGHTLY_TAG || (tag.starts_with('v') && parse_version(tag).is_some()) {
        Ok(())
    } else {
        Err(KobanError::Update {
            message: format!("invalid release tag `{tag}`; expected a tag like v0.1.0 or nightly"),
        })
    }
}

fn detect_install_kind(exe_path: &Path) -> InstallKind {
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
    use std::{
        cell::{Cell, RefCell},
        collections::HashSet,
    };

    use serde_json::Value;

    #[test]
    fn version_parsing_and_comparison_accept_v_prefixes() {
        assert_eq!(parse_version("0.1.2"), Some((0, 1, 2)));
        assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
        assert!(is_newer("0.0.1", "v0.0.2"));
        assert!(!is_newer("0.0.2", "v0.0.1"));
        assert_eq!(parse_version("v1.2"), None);
        assert!(validate_tag("v1.2.3").is_ok());
        assert!(validate_tag("1.2.3").is_err());
        assert_eq!(normalize_tag("nightly"), "nightly");
        assert!(validate_tag("nightly").is_ok());
    }

    #[test]
    fn explicit_check_tag_does_not_require_network_or_curl() {
        let current = env!("CARGO_PKG_VERSION");
        let output = run(true, false, Some(format!("v{current}")), false).expect("check");
        assert!(output.contains(&format!("Current version: {current}")));
        assert!(output.contains("Up to date"), "got: {output}");
    }

    #[test]
    fn explicit_invalid_tag_is_rejected() {
        let error =
            run(true, false, Some("not-a-version".to_string()), false).expect_err("invalid tag");
        assert!(error.to_string().contains("invalid release tag"));
    }

    #[test]
    fn downgrade_requires_force_before_install_detection() {
        let error =
            run(false, false, Some("v0.0.0".to_string()), false).expect_err("downgrade guard");
        assert!(error.to_string().contains("refusing to downgrade"));
    }

    #[test]
    fn current_version_without_check_exits_before_install_detection() {
        let current = env!("CARGO_PKG_VERSION");
        let ops = FakeOps::new(
            &format!("v{current}"),
            PathBuf::from("/nix/store/koban"),
            b"unused",
        );

        let output =
            run_with_ops(&ops, false, false, Some(format!("v{current}")), false).expect("current");

        assert!(
            output.contains(&format!("Already up to date ({current})")),
            "got: {output}"
        );
        assert_eq!(ops.curl_checks.get(), 0);
        assert!(ops.replaced.borrow().is_none());
    }

    #[test]
    fn check_reports_older_explicit_versions_without_downgrading() {
        let ops = FakeOps::new("v0.0.0", PathBuf::from("/tmp/koban"), b"unused");

        let output =
            run_with_ops(&ops, true, false, Some("v0.0.0".to_string()), false).expect("check");

        assert!(
            output.contains(&format!(
                "Version 0.0.0 is available (current: {})",
                env!("CARGO_PKG_VERSION")
            )),
            "got: {output}"
        );
        assert!(ops.replaced.borrow().is_none());
    }

    #[test]
    fn latest_check_uses_redirect_without_downloading_assets() {
        // 9.9.9 is a sentinel that is always newer than the real crate version,
        // so this stays green across release bumps.
        let ops = FakeOps::new("v9.9.9", PathBuf::from("/tmp/koban"), b"unused");

        let output = run_with_ops(&ops, true, false, None, false).expect("latest check");

        assert!(
            output.contains("New version available: 9.9.9"),
            "got: {output}"
        );
        assert_eq!(ops.curl_checks.get(), 1);
        assert!(ops.fetches.borrow().is_empty());
        assert!(ops.replaced.borrow().is_none());
    }

    #[test]
    fn nightly_check_uses_rolling_release_without_latest_redirect() {
        let ops = FakeOps::new("v9.9.9", PathBuf::from("/tmp/koban"), b"unused");

        let output = run_with_ops(&ops, true, false, None, true).expect("nightly check");

        assert!(
            output.contains("Nightly build available from the nightly release"),
            "got: {output}"
        );
        assert_eq!(ops.curl_checks.get(), 0);
        assert!(ops.fetches.borrow().is_empty());
        assert!(ops.replaced.borrow().is_none());
    }

    #[test]
    fn managed_update_downloads_verifies_and_replaces_binary() {
        let expected_binary = b"fresh-koban";
        let archive = make_tarball(&[("bin/koban", expected_binary)]);
        let exe = tempfile::tempdir()
            .expect("temp dir")
            .path()
            .join("bin")
            .join("koban");
        std::fs::create_dir_all(exe.parent().expect("parent")).expect("bin dir");
        // 9.9.9 is always newer than the real crate version (an upgrade, not a
        // downgrade), so the update proceeds regardless of release bumps.
        let ops = FakeOps::new("v9.9.9", exe.clone(), &archive);

        let output =
            run_with_ops(&ops, false, false, Some("v9.9.9".to_string()), false).expect("update");

        assert!(
            output.contains(&format!("Updating: {} -> 9.9.9", env!("CARGO_PKG_VERSION"))),
            "got: {output}"
        );
        assert!(output.contains("Checksum verified"));
        assert_eq!(ops.curl_checks.get(), 1);
        assert_eq!(
            ops.replaced.borrow().as_deref(),
            Some(expected_binary.as_slice())
        );
        let fetches = ops.fetches.borrow();
        assert_eq!(fetches.len(), 2);
        assert!(fetches.iter().any(
            |url| url.ends_with("/v9.9.9/koban-aarch64-apple-darwin.tar.gz")
                || url.ends_with("/v9.9.9/koban-x86_64-apple-darwin.tar.gz")
                || url.ends_with("/v9.9.9/koban-x86_64-unknown-linux-gnu.tar.gz")
                || url.ends_with("/v9.9.9/koban-aarch64-unknown-linux-gnu.tar.gz")
        ));
        assert!(
            fetches
                .iter()
                .any(|url| url.ends_with("/v9.9.9/SHA256SUMS"))
        );
    }

    #[test]
    fn managed_nightly_downloads_from_nightly_release() {
        let expected_binary = b"nightly-koban";
        let archive = make_tarball(&[("bin/koban", expected_binary)]);
        let dir = tempfile::tempdir().expect("temp dir");
        let exe = dir.path().join("koban");
        std::fs::write(&exe, b"old").expect("old binary");
        let ops = FakeOps::new("v9.9.9", exe, &archive);

        let output = run_with_ops(&ops, false, false, None, true).expect("nightly update");

        assert!(
            output.contains(&format!(
                "Installing nightly: {} -> nightly",
                env!("CARGO_PKG_VERSION")
            )),
            "got: {output}"
        );
        assert_eq!(
            ops.replaced.borrow().as_deref(),
            Some(expected_binary.as_slice())
        );
        assert!(
            ops.fetches
                .borrow()
                .iter()
                .all(|url| url.contains("/releases/download/nightly/"))
        );
    }

    #[test]
    fn package_managed_installs_return_upgrade_guidance() {
        // 9.9.9 is an upgrade, so the run reaches install-kind detection (the
        // Nix-managed guard) instead of stopping at the downgrade guard.
        let ops = FakeOps::new(
            "v9.9.9",
            PathBuf::from("/nix/store/abc-koban/bin/koban"),
            b"unused",
        );

        let error = run_with_ops(&ops, false, false, Some("v9.9.9".to_string()), false)
            .expect_err("managed install");

        assert!(error.to_string().contains("installed via Nix"), "{error}");
        assert!(
            error.to_string().contains("nix profile upgrade koban"),
            "{error}"
        );
        assert_eq!(ops.curl_checks.get(), 0);
    }

    #[test]
    fn upgrade_hints_cover_supported_package_managers() {
        assert_eq!(InstallKind::Cargo.label(), "cargo");
        assert_eq!(InstallKind::Homebrew.label(), "Homebrew");
        assert_eq!(InstallKind::Managed.label(), "direct install");
        assert!(
            upgrade_hint(InstallKind::Cargo, "v1.2.3")
                .expect("cargo hint")
                .contains("--tag v1.2.3")
        );
        assert_eq!(
            upgrade_hint(InstallKind::Homebrew, "v1.2.3").as_deref(),
            Some("  brew upgrade koban")
        );
        assert!(upgrade_hint(InstallKind::Managed, "v1.2.3").is_none());
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

        let err = verify_checksum(&sums, "koban-test.tar.gz", b"different")
            .expect_err("checksum mismatch");
        assert!(err.to_string().contains("checksum mismatch"));
    }

    #[test]
    fn tarball_extraction_finds_koban_binary() {
        let expected = b"fake-koban";
        let archive = make_tarball(&[("README.md", b"docs"), ("bin/koban", expected)]);
        assert_eq!(
            extract_binary_from_tarball(&archive).expect("binary"),
            expected
        );

        let archive = make_tarball(&[("README.md", b"docs")]);
        let err = extract_binary_from_tarball(&archive).expect_err("missing binary");
        assert!(err.to_string().contains("binary not found"));
    }

    #[test]
    fn writable_install_dir_probe_accepts_existing_parent() {
        let dir = tempfile::tempdir().expect("temp dir");
        let exe = dir.path().join("koban");

        ensure_writable_install_dir(&exe).expect("writable directory");
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

    struct FakeOps {
        latest_tag: String,
        exe_path: PathBuf,
        archive: Vec<u8>,
        sums: String,
        curl_checks: Cell<u32>,
        fetches: RefCell<HashSet<String>>,
        replaced: RefCell<Option<Vec<u8>>>,
    }

    impl FakeOps {
        fn new(latest_tag: &str, exe_path: PathBuf, archive: &[u8]) -> Self {
            let asset_name = detect_asset_name().expect("supported test platform");
            let mut hasher = Sha256::new();
            hasher.update(archive);
            let sums = format!("{:x}  {asset_name}\n", hasher.finalize());

            Self {
                latest_tag: latest_tag.to_string(),
                exe_path,
                archive: archive.to_vec(),
                sums,
                curl_checks: Cell::new(0),
                fetches: RefCell::new(HashSet::new()),
                replaced: RefCell::new(None),
            }
        }
    }

    impl UpdateOps for FakeOps {
        fn ensure_curl(&self) -> Result<()> {
            self.curl_checks.set(self.curl_checks.get() + 1);
            Ok(())
        }

        fn fetch_latest_tag(&self) -> Result<String> {
            Ok(self.latest_tag.clone())
        }

        fn current_exe(&self) -> Result<PathBuf> {
            Ok(self.exe_path.clone())
        }

        fn fetch_url(&self, url: &str) -> Result<Vec<u8>> {
            self.fetches.borrow_mut().insert(url.to_string());
            if url.ends_with("/SHA256SUMS") {
                Ok(self.sums.as_bytes().to_vec())
            } else {
                Ok(self.archive.clone())
            }
        }

        fn replace_binary(&self, new_binary: &[u8], _exe_path: &Path) -> Result<()> {
            *self.replaced.borrow_mut() = Some(new_binary.to_vec());
            Ok(())
        }
    }
}
