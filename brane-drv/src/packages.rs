use anyhow::Result;
use specifications::package::{PackageInfo, PackageIndex};
use serde_json::{json, Value as JValue};
use std::path::PathBuf;
use std::fs;
use semver::Version;

///
///
///
pub async fn get_package_index() -> Result<PackageIndex> {
    let packages_dir = get_packages_dir();

    let packages: JValue = if packages_dir.exists() {
        let packages_dir = get_packages_dir();
        if !packages_dir.exists() {
            return Ok(PackageIndex::empty());
        }

        let mut package_infos = Vec::<PackageInfo>::new();

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

                if let Ok(package_info) = PackageInfo::from_path(package_file) {
                    package_infos.push(package_info);
                }
            }
        }

        json!(package_infos)
    } else {
        todo!();
    };

    PackageIndex::from_value(packages)
}

///
///
///
pub fn get_package_dir(
    name: &str,
    version: Option<&str>,
) -> Result<PathBuf> {
    let packages_dir = get_packages_dir();
    let package_dir = packages_dir.join(&name);

    if version.is_none() {
        return Ok(package_dir);
    }

    let version = version.unwrap();
    let version = if version == "latest" {
        if !package_dir.exists() {
            return Err(anyhow!("Package not found."));
        }

        let versions = fs::read_dir(&package_dir)?;
        let mut versions: Vec<Version> = versions
            .map(|v| v.unwrap().file_name())
            .map(|v| Version::parse(&v.into_string().unwrap()).unwrap())
            .collect();

        versions.sort();
        versions.reverse();

        versions[0].to_string()
    } else {
        Version::parse(&version)
            .expect("Not a valid semantic version.")
            .to_string()
    };

    Ok(package_dir.join(version))
}

///
///
///
pub fn get_packages_dir() -> PathBuf {
    appdirs::user_data_dir(Some("brane"), None, false)
        .expect("Couldn't determine Brane data directory.")
        .join("packages")
}
