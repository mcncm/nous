#[macro_use]
extern crate erased_serde;

use std::fs;
use std::io;
use std::env;
use std::error::Error;
use std::str::FromStr;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};
//use std::hash::{Hash, Hasher};

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
            origin: None,
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
    pub fn new(name: String) -> Self {
        Project {
            nous_file: File{origin: None,
                            // TODO unwrap; env shouldn't be in here
                            dest_dir: env::current_dir().unwrap(),
                            name: name.clone() + ".nous",
            },
            resources: vec![],
            name,
        }
    }

    pub fn push<T: 'static + Fetchable>(&mut self, res: T) {
        self.resources.push(Box::new(res));
    }

    /// Instantiates a Project struct from a path: if this is a path to a
    /// nousfile, simply instantiate that project. If it's enclosed in a nous
    /// project, return that project.
    /*
    pub fn from_path(path: PathBuf) -> Self {
        // TODO
    }
    */

    /// Write out a .nous file
    fn write_to_file(&self, path: PathBuf) -> io::Result<()> {
        dbg!("Writing to file!");
        println!("Path is {:?}", path);
        let j = serde_json::to_string(&self)?;
        // let mut file = fs::File::open(&path)?;
        fs::write(path, j.as_bytes())?;
        Ok(())
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
    pub origin: Option<Address>,
    pub dest_dir: PathBuf,
    pub name: String,
}

impl Fetchable for GitRepository {
    fn fetch(&self) -> Result<(), Box<dyn Error>> {
        // TODO check for overwrite
        let dest_path = &self.dest_dir.as_path().join(&self.name);
        match &self.origin {
            Some(Address::Local(path)) => {
                symlink(path, dest_path)?;
            },
            Some(Address::RemoteHttp(url)) => {
                dbg!("Cloning git repository...");
                // TODO error conversion
                Repository::clone(url.as_str(), dest_path).unwrap();
            },
            None => {
                unimplemented!();
            }
        };

        Ok(())
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct File {
    pub origin: Option<Address>,
    pub dest_dir: PathBuf,
    pub name: String,
}

impl Fetchable for File {
    fn fetch(&self) -> Result<(), Box<dyn Error>> {
        // TODO check for overwrite
        let dest_path = &self.dest_dir.as_path().join(&self.name);
        match &self.origin {
            Some(Address::Local(path)) => {
                symlink(path, dest_path)?;
            },
            Some(Address::RemoteHttp(url)) => {
                dbg!("Attempting to download file");
                // TODO error conversion
                let mut response = reqwest::get(url.as_str()).unwrap();
                // TODO handle(/convert?) a reqwest error
                let mut dest = fs::File::create(dest_path).unwrap();
                io::copy(&mut response, &mut dest)?;
            },
            None => {
                unimplemented!();
            }
        };

        Ok(())
    }
}


fn validate_nous_repo(dir: &PathBuf) -> io::Result<()> {
    let md = fs::metadata(&dir)?;
    if md.is_dir() {
        let mut nous_path = dir.clone();
        nous_path.push(".nous");
        let md_nous = fs::metadata(&nous_path)?;
        if md_nous.is_file() {
            // TODO good enough for now, but should one day check contents
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound,
                               ".nous file not found"))
        }
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput,
                           format!("{} is not a directory", dir.to_str().unwrap())))
    }
}


/// Finds and returns the proximal nous repo contianing this path, if one exists.
fn enclosing_nous_repo(path: &PathBuf) -> Option<PathBuf> {
    // TODO consider adding a max recursion depth
    if is_nous_file(&path) {
        Some(path.parent().unwrap().to_path_buf())
    } else if let Some(parent) = path.parent() {
            let parent_buf = parent.to_path_buf();
            if is_nous_repo(&parent_buf){
                Some(parent_buf)
            } else{
                None
            }
    } else {
        None
    }
}


/// Checks if this path points to a nous repo diretory, returning `true` if it
/// is, and `false` otherwise.
fn is_nous_repo(dir: &PathBuf) -> bool {
    if let Ok(md) = fs::metadata(&dir) {
        if md.is_dir() {
            if let Ok(mut dir_entry) = fs::read_dir(dir) {
                dir_entry.any(|f| is_nous_file(&f.unwrap().path()))
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }  // TODO is this long else-false chain poor style?
}


/// Checks is this path points to a nous file, returning `true` if it
/// is, and `false` otherwise.
fn is_nous_file(file: &PathBuf) -> bool {
    if let Ok(md) = fs::metadata(&file) {
        if md.is_file() {
            file.file_name().unwrap().to_str() == Some(".nous")
        } else {
            false
        }
    } else {
        false
    }
}


/// Returns the path of a nous file contained in the given dir
fn nous_file_path(dir: &PathBuf) -> PathBuf {
    let mut nous_path = dir.clone();
    nous_path.push(".nous");
    nous_path
}

/// Initializes a nous repository in the specified directory.
pub fn nous_init(dir: PathBuf) -> io::Result<()> {
    if is_nous_repo(&dir) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
                   format!("{} is already a nous repo", dir.to_str().unwrap())))
    }
    let md = fs::metadata(&dir)?;
    if md.is_dir() {
        let nous_path = nous_file_path(&dir);
        // TODO this is pretty ugly, right?
        let proj = Project::new(dir.file_name().unwrap().to_str().unwrap().to_owned());
        proj.write_to_file(nous_path);
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput,
        format!("{} is not a directory", dir.to_str().unwrap())))
    }
}

/// Adds a resource to the enclosing repository, if one exists.
pub fn nous_add(res: PathBuf, resource: Box<dyn Fetchable>) -> io::Result<()> {
    // For now, just put it into the currrently enclosing dir--
    // This is subject to change, of course. This function should probably take
    // a dir parameter.
    if let Some(dir) = enclosing_nous_repo(&env::current_dir()?) {
        match validate_nous_repo(&dir) {
            Ok(()) => {
                // Deserialize the project
                let nous_path = nous_file_path(&dir);
                // let proj = Project::from_path(dir);
                // Add a new resource to it

                // Reserialize
                Ok(())
            },
            e => e,
        }
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput,
            format!("{} is not contained in a nous repo", res.to_str().unwrap())))
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
