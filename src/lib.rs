#[macro_use]
extern crate erased_serde;

use std::fs;
use std::io;
use std::env;
use std::error::Error;
use std::str::FromStr;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};
use serde_json;
use serde_traitobject;
// use std::hash::{Hash, Hasher};

use url::Url;
use git2::Repository;
use reqwest;  // TODO: why is this import unused? Code seems to work without import.


pub trait Fetchable: erased_serde::Serialize {
    fn fetch(&self) -> Result<(), Box<dyn Error>>;
}
serialize_trait_object!(Fetchable);

impl<'de> Deserialize<'de> for Box<dyn Fetchable> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /* dummy implementation */
        Ok(Box::new(File {
            origin: Address::new(String::new()),
            dest_dir: PathBuf::new(),
            name: String::new(),
        }))
    }
}

pub struct Config {
    symlink_local_resources: bool,
}


#[derive(Clone, Serialize, Deserialize)]
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


#[derive(Serialize, Deserialize)]
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

    // Instantiates a Project struct from the path of a nousfile
    /*
    pub fn from_nous_path(path: PathBuf) -> Self {
        // TODO
    }
    */

    /// Write out a .nous file
    fn write_file(&self, path: PathBuf) {
        // TODO
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


#[derive(Clone, Serialize, Deserialize)]
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


#[derive(Clone, Serialize, Deserialize)]
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


/*
/// From a uri and resource kind hint, attempt to create a Fetchable struct.
fn infer_resource(uri: String, maybe_kind: Option(String))
                      -> Option(Box<dyn Fetchable>) {

    // Open the project at dest_dir
    let addr = Address::new(uri);
    // Have to make best guess what kind of resource this is.
    match addr {
        Address::Local(path) => {
            // check if file, check if git repo
            let md = fs::metadata(&path);
            if md.is_file() {

            }
        },
        Address::RemoteHttp(path) => {
            if let Some(kind) = maybe_kind {
                match kind {
                    unimplemented!();
                }
            } else {
                eprintln!("Please specify a resource type for a remote resource.");
            }
        },
    }

    Ok(())
}
*/
