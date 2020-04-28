use chrono::Utc;
use console::{pad_str, Alignment};
use indicatif::HumanDuration;
use prettytable::format::FormatBuilder;
use prettytable::Table;
use specifications::groupmeta::GroupMeta as PackageInfo;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

type FResult<T> = Result<T, failure::Error>;

///
///
///
pub fn list() -> FResult<()> {
    let packages_dir = get_packages_dir();

    // Prepare display table.
    let format = FormatBuilder::new()
        .column_separator('\0')
        .borders('\0')
        .padding(1, 1)
        .build();

    let mut table = Table::new();
    table.set_format(format);
    table.add_row(row!["ID", "NAME", "VERSION", "CREATED"]);

    // Return early, if packages directory does not exist.
    if !packages_dir.exists() {
        table.print_tty(true);
        return Ok(());
    }

    // Add a row to the table for each version of each group.
    let packages = fs::read_dir(packages_dir)?;
    for package in packages {
        let package_path = package?.path();
        if !package_path.is_dir() {
            continue;
        }

        let versions = fs::read_dir(package_path)?;
        for version in versions {
            let path = version?.path();
            let package_file = path.join("package.yml");

            if !path.is_dir() || !package_file.exists() {
                continue;
            }

            let now = Utc::now().timestamp();
            if let Ok(package_info) = PackageInfo::from_path(package_file) {
                let uuid = format!("{}", &package_info.id);

                let id = pad_str(&uuid[..8], 10, Alignment::Left, Some(".."));
                let name = pad_str(&package_info.name, 20, Alignment::Left, Some(".."));
                let version = pad_str(&package_info.version, 15, Alignment::Left, Some(".."));
                let elapsed = Duration::from_secs((now - &package_info.created.timestamp()) as u64);
                let created = format!("{} ago", HumanDuration(elapsed));
                let created = pad_str(&created, 15, Alignment::Left, None);

                table.add_row(row![id, name, version, created]);
            }
        }
    }

    table.printstd();

    Ok(())
}

///
///
///
pub fn remove(_name: String) -> FResult<()> {
    println!("Remove package.");

    Ok(())
}

///
///
///
pub fn test(_name: String) -> FResult<()> {
    println!("Test package.");

    Ok(())
}

///
///
///
fn get_packages_dir() -> PathBuf {
    appdirs::user_data_dir(Some("brane"), None, false)
        .expect("Couldn't determine Brane data directory.")
        .join("packages")
}
