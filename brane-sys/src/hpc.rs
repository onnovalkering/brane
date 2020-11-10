use crate::System;
use anyhow::Result;
use std::path::PathBuf;
use url::Url;
use uuid::Uuid;
use std::env;
use std::sync::Arc;
use xenon_rs::storage::{FileSystem, FileSystemPath};
use xenon_rs::credentials::Credential;
use grpcio::{ChannelBuilder, EnvBuilder};

type Map<T> = std::collections::HashMap<String, T>;

lazy_static! {
    static ref HOSTNAME: String = env::var("HPC_HOSTNAME").unwrap_or_else(|_| String::from("slurm:22"));
    static ref FILE_SYTEM: String = env::var("HPC_FILESYSTEM").unwrap_or_else(|_| String::from("sftp"));
    static ref XENON: String = env::var("HPC_XENON").unwrap_or_else(|_| String::from("localhost:50051"));
    static ref DATA_DIR: String = env::var("HPC_DATA_DIR").unwrap_or_else(|_| String::from("/home/xenon"));

    // TODO: fetch credentials from vault (requires some refactoring to avoid circular dependencies).
    static ref USERNAME: String = env::var("HPC_USERNAME").unwrap_or_else(|_| String::from("xenon"));
    static ref PASSWORD: String = env::var("HPC_PASSWORD").unwrap_or_else(|_| String::from("javagat"));
}

///
///
///
#[derive(Clone)]
pub struct HpcSystem {
    uuid: Uuid,
}

impl HpcSystem {
    ///
    ///
    ///
    pub fn new(uuid: Uuid) -> Self {
        HpcSystem { uuid }
    }

    ///
    ///
    ///
    fn determine_parent(
        &self,
        uuid: &Uuid,
        parent: Option<&Url>,
        temp: bool,
    ) -> Result<PathBuf> {
        let parent = if let Some(parent) = parent {
            url_to_path(parent)?
        } else if temp {
            self.get_temp_dir(uuid)
        } else {
            self.get_session_dir(uuid)
        };

        Ok(parent)
    }

    ///
    ///
    ///
    fn get_session_dir(
        &self,
        uuid: &Uuid,
    ) -> PathBuf {
        PathBuf::from(DATA_DIR.as_str()).join("brane/sessions").join(uuid.to_string())
    }

    ///
    ///
    ///
    fn get_temp_dir(
        &self,
        uuid: &Uuid,
    ) -> PathBuf {
        PathBuf::from(DATA_DIR.as_str()).join("brane/temp").join(uuid.to_string())
    }

    ///
    ///
    ///
    pub fn create_filesystem(&self) -> Result<FileSystem> {
        let env = Arc::new(EnvBuilder::new().build());
        let channel = ChannelBuilder::new(env).connect(XENON.as_str());

        let credential = Credential::new_password(USERNAME.to_string(), PASSWORD.to_string());

        let mut properties = Map::<String>::new();
        properties.insert(
            String::from("xenon.adaptors.filesystems.sftp.strictHostKeyChecking"),
            String::from("false"),
        );

        Ok(FileSystem::create(
            FILE_SYTEM.to_string(),
            channel,
            credential,
            HOSTNAME.to_string(),
            properties,
        ).unwrap())
    }
}

impl System for HpcSystem {
    fn clone(&self) -> Box<dyn System> {
        let system = HpcSystem {
            uuid: self.uuid.clone(),
        };

        Box::new(system)
    }

    fn create_dir(
        &self,
        name: &str,
        parent: Option<&Url>,
        temp: bool,
    ) -> Result<Url> {
        let fs = self.create_filesystem()?;

        let parent = self.determine_parent(&self.uuid, parent, temp)?;
        let fs_parent = FileSystemPath::new(parent.clone().into_os_string().into_string().unwrap());
        if !fs.exists(fs_parent.clone()).unwrap() {
            fs.create_directories(fs_parent.clone()).unwrap();
        }


        let dir = parent.join(name);
        let fs_dir = FileSystemPath::new(dir.clone().into_os_string().into_string().unwrap());
        if !fs.exists(fs_dir.clone()).unwrap() {
            fs.create_directory(fs_dir.clone()).unwrap();
        }

        path_to_url(&dir)
    }

    fn create_file(
        &self,
        name: &str,
        parent: Option<&Url>,
        temp: bool,
    ) -> Result<Url> {
        let fs = self.create_filesystem()?;

        let parent = self.determine_parent(&self.uuid, parent, temp)?;
        let fs_parent = FileSystemPath::new(parent.clone().into_os_string().into_string().unwrap());
        if !fs.exists(fs_parent.clone()).unwrap() {
            fs.create_directories(fs_parent.clone()).unwrap();
        }

        let file = parent.join(name);
        let fs_file = FileSystemPath::new(file.clone().into_os_string().into_string().unwrap());
        if !fs.exists(fs_file.clone()).unwrap() {
            fs.create_file(fs_file.clone()).unwrap();
        }

        path_to_url(&file)
    }

    fn get_session_id(&self) -> Uuid {
        self.uuid.clone()
    }

    fn get_temp_dir(&self) -> PathBuf {
        self.get_temp_dir(&self.uuid)
    }

    fn get_session_dir(&self) -> PathBuf {
        self.get_session_dir(&self.uuid)
    }
}

///
///
///
fn path_to_url(path: &PathBuf) -> Result<Url> {
    Ok(Url::parse(&format!(
        "file://{}",
        path.clone().into_os_string().into_string().unwrap()
    ))?)
}

///
///
///
fn url_to_path(url: &Url) -> Result<PathBuf> {
    Ok(PathBuf::from(url.path()))
}

