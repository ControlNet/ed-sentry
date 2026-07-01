fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_resource::compile("resources/windows.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("ed-sentry launcher icon resource must compile");
    }
}
