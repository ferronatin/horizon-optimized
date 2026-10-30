//! Embeds Horizon's icon (`packaging/icons/horizon.ico`) in the Windows executables, as icon
//! resource 1: Explorer and the task bar show it, and the window takes it as its icon.
fn main() {
    // Renderer initialization runs on the window's main thread, including the
    // graphics driver's calls. MSVC's 1 MiB executable default is too small
    // for this path on legacy Windows OpenGL drivers. Reserve address space;
    // retain the default small commit so this does not eagerly consume RAM.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        println!("cargo:rustc-link-arg-bin=horizon-client=/STACK:16777216,4096");
    }
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let ico = format!("{dir}/../../packaging/icons/horizon.ico");
    println!("cargo:rerun-if-changed={ico}");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon(&ico);
        if let Err(e) = resource.compile() {
            panic!("embedding {ico}: {e}");
        }
    }
}
