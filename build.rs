fn main() {
    if !std::path::Path::new("cfg.toml").exists() {
        panic!("cfg.toml is missing from root directory");
    }

    embuild::espidf::sysenv::output();
}
