// SocialCount - Real-time YouTube Channel Statistics
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount

fn main() {
    #[cfg(windows)]
    {
        if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
            let mut res = winres::WindowsResource::new();
            res.set_icon("assets/icon.ico");
            res.set("ProductName", "SocialCount");
            res.set(
                "FileDescription",
                "SocialCount - Real-time YouTube Channel Statistics",
            );
            res.set("LegalCopyright", "Copyright (c) Matteo Baccan");
            if let Err(e) = res.compile() {
                eprintln!("cargo:warning=Failed to compile Windows resource: {e}");
            }
        }
    }
}
