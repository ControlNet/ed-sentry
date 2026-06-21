pub fn write_minimal_journal(folder: &std::path::Path) -> std::path::PathBuf {
    write_named_journal(folder, "Journal.2035-01-03T100000.01.log", "Minimal Raider")
}

pub fn write_named_journal(
    folder: &std::path::Path,
    filename: &str,
    ship_name: &str,
) -> std::path::PathBuf {
    let journal_path = folder.join(filename);
    std::fs::write(
        &journal_path,
        format!(
            concat!(
                r#"{{"timestamp":"2035-01-03T10:00:00Z","event":"Fileheader"}}"#,
                "\n",
                r#"{{"timestamp":"2035-01-03T10:02:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"{}","PilotName":"Fixture Pirate","LegalStatus":"Wanted"}}"#,
                "\n"
            ),
            ship_name
        ),
    )
    .unwrap();
    journal_path
}
