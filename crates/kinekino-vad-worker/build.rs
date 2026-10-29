fn main() {
    println!("cargo:rerun-if-env-changed=ORT_LIB_PATH");
    if std::env::var_os("CARGO_FEATURE_NATIVE_ORT").is_some() {
        assert!(
            std::env::var_os("ORT_LIB_PATH").is_some(),
            "native-ort requires the centrally approved pinned ORT_LIB_PATH; no download fallback"
        );
        let target = std::env::var("TARGET").expect("Cargo target");
        assert_eq!(target, "x86_64-pc-windows-msvc", "Unverified native target");
        // Same existing static distribution import as frozen r3; not DML EP use.
        println!("cargo:rustc-link-lib=DirectML");
    }
}
