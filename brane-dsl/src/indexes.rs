use anyhow::Result;
use semver::Version;
use serde_json::Value as JValue;
use specifications::package::PackageInfo;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

type Map<T> = std::collections::HashMap<String, T>;

#[derive(Debug)]
pub struct PackageIndex {
    pub packages: Map<PackageInfo>,
    pub standard: Map<PackageInfo>,
    pub versions: Map<Vec<Version>>,
}

impl PackageIndex {
    ///
    ///
    ///
    pub fn empty() -> Self {
        let packages = Map::<PackageInfo>::new();
        let versions = Map::<Vec<Version>>::new();

        PackageIndex::new(packages, versions)
    }

    ///
    ///
    ///
    pub fn new(
        packages: Map<PackageInfo>,
        mut versions: Map<Vec<Version>>,
    ) -> Self {
        // Make sure the latest version can be retrieved with .first()
        for (_, p_versions) in versions.iter_mut() {
            p_versions.sort();
            p_versions.reverse();
        }

        let standard = brane_std::PACKAGES.clone();
        PackageIndex { packages, standard, versions }
    }

    ///
    ///
    ///
    pub fn from_path(path: &PathBuf) -> Result<Self> {
        let file = File::open(path)?;
        let buf_reader = BufReader::new(file);

        PackageIndex::from_reader(buf_reader)
    }

    ///
    ///
    ///
    pub fn from_reader<R: Read>(r: R) -> Result<Self> {
        let v = serde_json::from_reader(r)?;

        PackageIndex::from_value(v)
    }

    ///
    ///
    ///
    pub fn from_value(v: JValue) -> Result<Self> {
        let known_packages: Vec<PackageInfo> = serde_json::from_value(v)?;

        let mut packages = Map::<PackageInfo>::new();
        let mut versions = Map::<Vec<Version>>::new();
        for package in known_packages {
            let key = format!("{}-{}", package.name, package.version);
            packages.insert(key, package.clone());

            let version = Version::parse(&package.version)?;
            if let Some(p_versions) = versions.get_mut(&package.name) {
                p_versions.push(version);
            } else {
                versions.insert(package.name, vec![version]);
            }
        }

        Ok(PackageIndex::new(packages, versions))
    }

    ///
    ///
    ///
    pub fn get(
        &self,
        name: &str,
        version: Option<&Version>,
    ) -> Option<&PackageInfo> {
        let standard_package = self.standard.get(name);
        if standard_package.is_some() {
            return standard_package;
        }

        let version = if let None = version {
            if let Some(version) = self.get_latest_version(name) {
                version
            } else {
                return None
            }
        } else {
            version.unwrap()
        };

        self.packages.get(&format!("{}-{}", name, version))
    }

    ///
    ///
    ///
    fn get_latest_version(
        &self,
        name: &str,
    ) -> Option<&Version> {
        self.versions.get(name).map(|vs| vs.first()).unwrap_or(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_packages() {
        let packages_json = PathBuf::from("./resources/packages.json");
        let result = PackageIndex::from_path(&packages_json);

        assert!(result.is_ok());
    }
}
