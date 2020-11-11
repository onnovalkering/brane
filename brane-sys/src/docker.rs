use crate::System;
use anyhow::Result;
use std::fs::{self, File};
use std::path::PathBuf;
use url::Url;
use uuid::Uuid;

///
///
///
#[derive(Clone)]
pub struct DockerSystem {
    uuid: Uuid,
}

impl DockerSystem {
    ///
    ///
    ///
    pub fn new(uuid: Uuid) -> Self {
        DockerSystem { uuid }
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
        PathBuf::from("/brane/session").join(uuid.to_string())
    }

    ///
    ///
    ///
    fn get_temp_dir(
        &self,
        uuid: &Uuid,
    ) -> PathBuf {
        PathBuf::from("/brane/temp").join(uuid.to_string())
    }
}

impl System for DockerSystem {
    fn clone(&self) -> Box<dyn System> {
        let system = DockerSystem {
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
        let parent = self.determine_parent(&self.uuid, parent, temp)?;
        debug!("Creating dirs: {:?}", parent);
        fs::create_dir_all(&parent)?;

        let dir = parent.join(name);
        if !dir.exists() {
            debug!("Creating dir: {:?}", dir);
            fs::create_dir(&dir)?;
        }

        path_to_url(&dir)
    }

    fn create_file(
        &self,
        name: &str,
        parent: Option<&Url>,
        temp: bool,
    ) -> Result<Url> {
        let parent = self.determine_parent(&self.uuid, parent, temp)?;
        fs::create_dir_all(&parent)?;

        let file = parent.join(name);
        File::create(&file)?;

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
