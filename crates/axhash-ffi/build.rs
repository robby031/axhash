fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").expect("missing manifest dir");
    let out_file = std::path::Path::new(&crate_dir).join("include").join("axhash.h");

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(cbindgen::Config::from_file("cbindgen.toml").expect("cbindgen.toml"))
        .generate()
        .expect("unable to generate bindings")
        .write_to_file(out_file);
}
