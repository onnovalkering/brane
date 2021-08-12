use crate::common::{Function, Type};
use anyhow::Result;
use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value as JValue;
use serde_with::skip_serializing_none;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use uuid::Uuid;

type Map<T> = std::collections::HashMap<String, T>;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageInfo {
    pub created: DateTime<Utc>,
    pub description: String,
    pub detached: bool,
    pub functions: Option<Map<Function>>,
    pub id: Uuid,
    pub kind: String,
    pub name: String,
    pub owners: Vec<String>,
    pub types: Option<Map<Type>>,
    pub version: String,
}

#[allow(unused)]
impl PackageInfo {
    ///
    ///
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        version: String,
        description: String,
        detached: bool,
        kind: String,
        owners: Vec<String>,
        functions: Option<Map<Function>>,
        types: Option<Map<Type>>,
    ) -> PackageInfo {
        let id = Uuid::new_v4();
        let created = Utc::now();

        PackageInfo {
            created,
            description,
            detached,
            functions,
            id,
            kind,
            name,
            owners,
            types,
            version,
        }
    }

    pub fn from_path(path: PathBuf) -> Result<PackageInfo> {
        let contents = fs::read_to_string(path)?;

        PackageInfo::from_string(contents)
    }

    pub fn from_string(contents: String) -> Result<PackageInfo> {
        let result = serde_yaml::from_str(&contents)?;

        Ok(result)
    }
}

#[derive(Debug, Clone, Default)]
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

        let standard = Map::default();
        PackageIndex {
            packages,
            standard,
            versions,
        }
    }

    ///
    ///
    ///
    pub fn from_path(path: &Path) -> Result<Self> {
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
    pub async fn from_url(url: &str) -> Result<Self> {
        let json = reqwest::get(url).await?.json().await?;

        PackageIndex::from_value(json)
    }

    ///
    ///
    ///
    pub fn from_value(v: JValue) -> Result<Self> {
        let known_packages: Vec<PackageInfo> = serde_json::from_value(v)?;
        PackageIndex::from_packages(known_packages)
    }

    ///
    ///
    ///
    pub fn from_packages(known_packages: Vec<PackageInfo>) -> Result<Self> {
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

        let version = if version.is_none() {
            self.get_latest_version(name)?
        } else {
            version?
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
