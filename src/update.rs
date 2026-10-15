//! `todoer update` from the GitHub releases, and a check for a new version once a day.

use crate::VERSION;
use crate::colors;
use crate::logger;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const RELEASES_URL: &str = "https://github.com/sobanieca/todoer/releases";
pub const NO_UPDATE_CHECK_ENV: &str = "TODOER_NO_UPDATE_CHECK";
const RELEASES_URL_ENV: &str = "TODOER_RELEASES_URL";
const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

fn releases_url() -> String {
    std::env::var(RELEASES_URL_ENV)
        .ok()
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| RELEASES_URL.to_string())
}

fn asset_name() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Some("todoer-linux-x64"),
        ("linux", "aarch64") => Some("todoer-linux-arm64"),
        ("macos", "x86_64") => Some("todoer-macos-x64"),
        ("macos", "aarch64") => Some("todoer-macos-arm64"),
        _ => None,
    }
}

fn agent(timeout: Option<Duration>, max_redirects: u32) -> ureq::Agent {
    ureq::Agent::config_builder()
        .max_redirects(max_redirects)
        .timeout_global(timeout)
        .user_agent(format!("todoer/{VERSION}"))
        .build()
        .new_agent()
}

fn latest_version(timeout: Option<Duration>) -> Result<String, String> {
    let url = format!("{}/latest", releases_url());
    let response = agent(timeout, 0)
        .get(&url)
        .call()
        .map_err(|err| format!("cannot check the latest version: {err}"))?;
    let location = response
        .headers()
        .get("location")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| "cannot check the latest version: there is no release".to_string())?;
    let tag = location
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or_default();
    let version = tag.strip_prefix('v').unwrap_or(tag);
    if parse_version(version).is_none() {
        return Err(format!(
            "cannot check the latest version: the release tag {tag} is not a version"
        ));
    }
    Ok(version.to_string())
}

fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let core = version.split(['-', '+']).next()?;
    let mut parts = core.split('.').map(|part| part.parse::<u64>().ok());
    Some((parts.next()??, parts.next()??, parts.next()??))
}

fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(latest), Some(current)) => latest > current,
        _ => false,
    }
}

fn install_hint(path: &Path) -> String {
    format!(
        "no permission to replace {} - run sudo todoer update",
        path.display()
    )
}

fn map_io(err: std::io::Error, exe: &Path) -> String {
    if err.kind() == std::io::ErrorKind::PermissionDenied {
        install_hint(exe)
    } else {
        format!("cannot replace {}: {err}", exe.display())
    }
}

pub fn execute() -> Result<(), String> {
    logger::info("checking for updates...");
    let latest = latest_version(Some(Duration::from_secs(30)))?;
    write_cache(&latest);
    if !is_newer(&latest, VERSION) {
        logger::info(&format!("todoer is up to date ({VERSION})"));
        return Ok(());
    }

    let asset = asset_name().ok_or_else(|| {
        format!(
            "todoer update does not support {}-{} - download todoer from {RELEASES_URL}/latest",
            std::env::consts::OS,
            std::env::consts::ARCH
        )
    })?;
    let exe = std::env::current_exe()
        .and_then(|path| path.canonicalize())
        .map_err(|err| format!("cannot find the todoer binary: {err}"))?;

    logger::info(&format!("updating todoer {VERSION} -> {latest}..."));
    let url = format!("{}/download/v{latest}/{asset}", releases_url());
    let mut response = agent(Some(Duration::from_secs(300)), 10)
        .get(&url)
        .call()
        .map_err(|err| format!("cannot download {url}: {err}"))?;
    let binary = response
        .body_mut()
        .with_config()
        .limit(u64::MAX)
        .read_to_vec()
        .map_err(|err| format!("cannot download {url}: {err}"))?;

    let temp = exe.with_file_name(format!(".todoer-update-{}", std::process::id()));
    let result = install(&temp, &exe, &binary, &latest);
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result?;

    logger::info(&format!("todoer is now {latest}"));
    Ok(())
}

fn install(temp: &Path, exe: &Path, binary: &[u8], version: &str) -> Result<(), String> {
    std::fs::write(temp, binary).map_err(|err| map_io(err, exe))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(temp, std::fs::Permissions::from_mode(0o755))
            .map_err(|err| map_io(err, exe))?;
    }

    let output = std::process::Command::new(temp)
        .arg("--version")
        .env(NO_UPDATE_CHECK_ENV, "1")
        .output()
        .map_err(|err| format!("cannot start the downloaded binary: {err}"))?;
    let reported = colors::strip(String::from_utf8_lossy(&output.stdout).trim());
    let reported = reported
        .strip_prefix("todoer ")
        .unwrap_or(&reported)
        .to_string();
    if reported != version {
        return Err(format!(
            "the downloaded binary is version {reported}, not {version} - nothing changed"
        ));
    }

    std::fs::rename(temp, exe).map_err(|err| map_io(err, exe))
}

fn cache_file() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|home| !home.is_empty())
                .map(|home| Path::new(&home).join(".cache"))
        })?;
    Some(base.join("todoer").join("latest-version"))
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn read_cache() -> Option<(u64, String)> {
    let content = std::fs::read_to_string(cache_file()?).ok()?;
    let mut lines = content.lines();
    let checked_at = lines.next()?.parse().ok()?;
    Some((checked_at, lines.next().unwrap_or_default().to_string()))
}

fn write_cache(version: &str) {
    let Some(path) = cache_file() else { return };
    let Some(dir) = path.parent() else { return };
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    let temp = dir.join(format!(".latest-version-{}", std::process::id()));
    if std::fs::write(&temp, format!("{}\n{version}\n", now())).is_ok() {
        let _ = std::fs::rename(&temp, &path);
    }
}

pub struct UpdateCheck(mpsc::Receiver<Option<String>>);

pub fn start_background_check() -> Option<UpdateCheck> {
    if std::env::var_os(NO_UPDATE_CHECK_ENV).is_some_and(|v| !v.is_empty())
        || !std::io::stderr().is_terminal()
    {
        return None;
    }
    if let Some((checked_at, _)) = read_cache()
        && now().saturating_sub(checked_at) < CHECK_INTERVAL.as_secs()
    {
        return None;
    }
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let latest = latest_version(Some(Duration::from_secs(3))).ok();
        write_cache(latest.as_deref().unwrap_or_default());
        let _ = sender.send(latest);
    });
    Some(UpdateCheck(receiver))
}

pub fn notify(check: Option<UpdateCheck>) {
    let Some(UpdateCheck(receiver)) = check else {
        return;
    };
    let Ok(Some(latest)) = receiver.recv_timeout(Duration::from_millis(500)) else {
        return;
    };
    if is_newer(&latest, VERSION) {
        eprintln!(
            "{}",
            colors::yellow(&format!(
                "\na new version of todoer is available: {VERSION} -> {latest}\nrun todoer update to install it"
            ))
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(is_newer("5.0.1", "5.0.0"));
        assert!(is_newer("5.1.0", "5.0.9"));
        assert!(is_newer("10.0.0", "9.9.9"));
        assert!(!is_newer("5.0.0", "5.0.0"));
        assert!(!is_newer("4.9.9", "5.0.0"));
        assert!(!is_newer("garbage", "5.0.0"));
        assert_eq!(parse_version("5.0.0-beta.1"), Some((5, 0, 0)));
    }
}
