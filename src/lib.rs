use std::fs;
use std::io;
use std::env;
use std::error::Error;
use std::str::FromStr;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
// use std::hash::{Hash, Hasher};

use url::Url;
use git2::Repository;
use reqwest;  // TODO: why is this import unused? Code seems to work without import.


pub trait Fetchable {
    fn fetch(&self) -> Result<(), Box<dyn Error>>;
}


pub struct Config {
    symlink_local_resources: bool,
}


#[derive(Clone)]
pub enum Address {
    Local(PathBuf),
    RemoteHttp(Url),
}

impl Address {
    pub fn new(uri: String) -> Self {
        // Figure out if local or remote as stupidly as possible
        let uri_wrap = Url::from_str(&uri);
        match uri_wrap {
            Ok(url) => Address::RemoteHttp(url),
            Err(_) => {
                // Assume that it must be a local resource
                Address::Local(PathBuf::from(&uri))
            },
        }
    }
}


pub struct Project {
    nous_file: File,
    resources: Vec<Box<dyn Fetchable>>,
    name: String,
}


/// for now, the format for .nous files will be a simple .json list
/// [{name: 'name', kind: 'kind', origin: 'origin'}]
///
/// in the future, this will get /much/ more complicated, and will be the
/// epicenter of essential complexity in the application.
impl Project {
    pub fn new(name: String, origin: Address) -> Self {
        Project {
            nous_file: File{origin: origin,
                            // TODO unwrap
                            dest_dir: env::current_dir().unwrap(),
                            name: name.clone() + ".nous",
            },
            resources: vec![],
            name: name,
        }
    }

    pub fn push<T: 'static + Fetchable>(&mut self, res: T) {
        self.resources.push(Box::new(res));
    }

    pub fn write_file(&self) {
        // Write out a .nous file
    }
}


impl Fetchable for Project {
    /// Retrieve the .nous file and all contained resources
    fn fetch(&self) -> Result<(), Box<dyn Error>> {
        // First, retrieve the .nous file to a temp file.
        self.nous_file.fetch();
        // Read the .nous file, populate resources.

        // Make local directory, move .nous into it and cwd.

        // Fetch resources.

        Ok(())
    }
}


#[derive(Clone)]
pub struct GitRepository {
    pub origin: Address,
    pub dest_dir: PathBuf,
    pub name: String,
}

impl Fetchable for GitRepository {
    fn fetch(&self) -> Result<(), Box<dyn Error>> {
        // TODO check for overwrite
        let dest_path = &self.dest_dir.as_path().join(&self.name);
        match &self.origin {
            Address::Local(path) => {
                symlink(path, dest_path)?;
            },
            Address::RemoteHttp(url) => {
                dbg!("Cloning git repository...");
                // TODO error conversion
                Repository::clone(url.as_str(), dest_path).unwrap();
            },
        };

        Ok(())
    }
}


#[derive(Clone)]
pub struct File {
    pub origin: Address,
    pub dest_dir: PathBuf,
    pub name: String,
}

impl Fetchable for File {
    fn fetch(&self) -> Result<(), Box<dyn Error>> {
        // TODO check for overwrite
        let dest_path = &self.dest_dir.as_path().join(&self.name);
        match &self.origin {
            Address::Local(path) => {
                symlink(path, dest_path)?;
            },
            Address::RemoteHttp(url) => {
                dbg!("Attempting to download file");
                // TODO error conversion
                let mut response = reqwest::get(url.as_str()).unwrap();
                // TODO handle(/convert?) a reqwest error
                let mut dest = fs::File::create(dest_path).unwrap();
                io::copy(&mut response, &mut dest)?;
            },
        };

        Ok(())
    }
}
