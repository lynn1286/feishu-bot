fn main() {
    let pkg = std::fs::read_to_string("package.json").unwrap();
    let version = pkg.split("\"version\":").nth(1).unwrap()
        .split('"').nth(1).unwrap();
    println!("cargo:rustc-env=PKG_VERSION={}", version);
}
