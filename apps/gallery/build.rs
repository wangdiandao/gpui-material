fn main() {
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var_os("CARGO_FEATURE_DESKTOP").is_some()
    {
        println!("cargo:rerun-if-changed=gallery.rc");
        println!("cargo:rerun-if-changed=assets/gallery.ico");

        embed_resource::compile_for(
            "gallery.rc",
            ["gpui-material-gallery"],
            embed_resource::NONE,
        )
        .manifest_required()
        .expect("embed the Gallery application icon");
    }
}
