use std::fs;
use std::io;
use std::env;
use std::error::Error;
use std::str::FromStr;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
// use crypto::digest::Digest;
use url::Url;
use git2::Repository;
use reqwest;


#[typetag::serde]
pub trait Fetchable {
    fn fetch(&self) -> Result<(), Box<dyn Error>>;
}


/* 
pub struct Config {
    symlink_local_resources: bool,
}
*/


#[derive(Clone, Serialize, Deserialize)]
pub enum Address {
    Local(PathBuf),
    RemoteHttp(Url),
}


impl Address {
    pub fn from_string(uri: String) -> Self {
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
    nous_file: NousFile,
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
            nous_file: NousFile{
                origin: None,
                // TODO unwrap; env shouldn't be in here
                dest_dir: env::current_dir().unwrap(),
                name: name.clone() + ".nous",
            },
            resources: vec![],
            name,
        }
    }

    pub fn push(&mut self, res: Box<dyn Fetchable>) {
        self.resources.push(res);
    }

    /// Read a project from a .nous file
    fn from_file(path: &PathBuf) -> Option<Self> {
        if let Ok(j) = fs::read(&path.as_path()) {
            println!("Successfully loaded json");
            if let Ok(proj) = serde_json::from_slice(&j[..]) {
                println!("Successfully deserialized json");
                Some(proj)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Write out a .nous file
    fn write_to_file(&self, path: &PathBuf) -> io::Result<()> {
        dbg!("Writing to file!");
        println!("Path is {:?}", path);
        let j = serde_json::to_string(&self)?;
        // let mut file = fs::File::open(&path)?;
        fs::write(path, j.as_bytes())?;
        Ok(())
    }
}

#[typetag::serde]
impl Fetchable for Project {
    /// Retrieve the .nous file and all contained resources
    fn fetch(&self) -> Result<(), Box<dyn Error>> {
        // First, retrieve the .nous file to a temp file.
        //self.nous_file.fetch();
        // Read the .nous file, populate resources.

        // Make local directory, move .nous into it and cwd.

        // Fetch resources.

        unimplemented!();
        //Ok(())
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct GitRepository {
    pub origin: Option<Address>,
    pub dest_dir: PathBuf,
    pub name: String,
}

#[typetag::serde]
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
pub struct NousFile {
    pub origin: Option<Address>,
    pub dest_dir: PathBuf,
    pub name: String,
}


// MAJOR TODO: should be able to derive this

#[typetag::serde]
impl Fetchable for NousFile {
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


#[derive(Clone, Serialize, Deserialize)]
pub struct File {
    pub origin: Option<Address>,
    pub dest_dir: PathBuf,
    pub name: String,
    pub hash: Vec<u8>,
}

#[typetag::serde]
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


/// finds and returns the proximal nous repo containing and including this path,
/// if one exists.
fn enclosing_nous_repo_incl(path: &PathBuf) -> Option<PathBuf> {
    if is_nous_repo(&path) {
        Some(path.to_owned())
    } else {
        println!("I made it here with path {:?}", &path);
        enclosing_nous_repo(path)
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
    // TODO doesn't validate that this file exists
    let mut nous_path = dir.clone();
    nous_path.push(".nous");
    nous_path
}


/// Initializes a nous repository in the specified directory.
pub fn nous_init(dir: &PathBuf) -> io::Result<()> {
    if is_nous_repo(&dir) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
                   format!("{} is already a nous repo", dir.to_str().unwrap())))
    }
    let md = fs::metadata(&dir)?;
    if md.is_dir() {
        let nous_path = nous_file_path(&dir);
        // TODO this is pretty ugly, right?
        let proj = Project::new(dir.file_name().unwrap().to_str().unwrap().to_owned());
        proj.write_to_file(&nous_path)?;
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput,
        format!("{} is not a directory", dir.to_str().unwrap())))
    }
}


/// Adds a resource inferred from a uri name
pub fn nous_add(uri: String) -> io::Result<()> {
    if let Some(res) = infer_resource(uri) {
        nous_add_resource(res)?;
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput,
            "I don't understand the meaning of that resource!"))
    }
}


/// Adds a resource to the enclosing repository, if one exists.
fn nous_add_resource(resource: Box<dyn Fetchable>) -> io::Result<()> {
    // For now, just put it into the currrently enclosing dir--
    // This is subject to change, of course. This function should probably take
    // a dir parameter.
    if let Some(repo) = enclosing_nous_repo_incl(&env::current_dir()?) {
        match validate_nous_repo(&repo) {
            Ok(()) => {
                let nous_file = nous_file_path(&repo);
                // Deserialize the project
                dbg!(&repo, &nous_file);
                let mut proj = Project::from_file(&nous_file).unwrap();
                // TODO Add a new resource to it
                proj.push(resource);
                // Reserialize
                proj.write_to_file(&nous_file)
            },
            e => e,
        }
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput,
            "working directory not contained in a nous repo"))
    }
}


/// From a uri and resource kind hint, attempt to create a Fetchable struct.
fn infer_resource(uri: String) -> Option<Box<dyn Fetchable>> {

    // Open the project at dest_dir
    let addr = Address::from_string(uri);
    // Have to make best guess what kind of resource this is.
    match &addr {
        Address::Local(path) => {
            // check if file, check if git repo
            if let Ok(md) = fs::metadata(&path) {
                if md.is_file() {
                    Some(Box::new(File {
                        origin: Some(addr.clone()),
                        dest_dir: PathBuf::from_str(
                            path.file_stem().unwrap().to_str().unwrap()
                        ).unwrap(),
                        name: path.file_name().unwrap().to_str().unwrap().to_owned(),
                        hash: hash_file(&path),
                    }))

                // TODO is there a better/faster way to do this with git2, or roll my own?
                } else if git2::Repository::open(&path).is_ok() {
                    Some(Box::new(GitRepository {
                        origin: Some(addr.clone()),
                        dest_dir: PathBuf::from_str(
                            path.file_stem().unwrap().to_str().unwrap()
                        ).unwrap(),
                        name: path.file_name().unwrap().to_str().unwrap().to_owned(),
                    }))
                } else {
                    None
                }
            } else {
                None
            }
        },

        _ => { unimplemented!(); },
        /*
        Address::RemoteHttp(path) => {
            unimplemented!();
            if let Some(kind) = maybe_kind {
                match kind {
                    unimplemented!();
                }
            } else {
                eprintln!("Please specify a resource type for a remote resource.");
            }
        },
        */
    }
}

/// Calculate a checksum for a single file. Has to return a byte vector,
/// rathter than a byte array, so size is known at compile time.
fn hash_file(path: &PathBuf) -> Vec<u8> {
    let mut file = fs::File::open(path).unwrap();
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).unwrap();
    let hash = hasher.result().iter().map(|u| u.to_owned()).collect();
    println!("hashed file contents are: {:?}", hash);
    hash
}


#[cfg(test)]
mod tests{
    use super::*;

    // Common variables that appear in all of the tests
    #[macro_export]
    macro_rules! setup_test_structs {
        // TODO: be more careful, e.g. to use a tempdir with a guaranteed-
        // unique name
        ($var1:ident, $var2:ident, $var3:ident, $var4:ident) => {
            let mut $var1 = env::current_dir().unwrap(); // old dir
            let mut $var2 = env::current_dir().unwrap().join("test_dir"); // dir
            let $var3 = nous_file_path(&$var2); // nous file
            let mut $var4 = $var2.clone().join("file.txt"); // test file
        };
    }

    // Make a test directory and create a repository in it.
    #[test]
    fn test_repo_roundtrip() {
        setup_test_structs!(old_dir, test_dir, nous_path, file_path);
        // Clean up
        fs::remove_dir_all(&test_dir).unwrap_or_else(drop);
        // Make a new one
        fs::create_dir(&test_dir).expect("Failed to create test directory.");
        nous_init(&test_dir);

        // Add a file to it.
        fs::File::create(&file_path).expect("Failed to create test file");
        env::set_current_dir(&test_dir).expect("Failed to change to test directory");
        nous_add(file_path.as_path().to_str().unwrap().to_owned())
            .expect("Failed to add file");

        // Read the .nous file.
        let mut json_1 = vec![];
        if let Ok(j) = fs::read(&nous_path.as_path()) {
            json_1 = j;
        }

        // Load it to a project and write it back. This
        dbg!(&nous_path);
        let proj = Project::from_file(&nous_path).expect("Failed to load proj");
        proj.write_to_file(&nous_path).expect("Failed to write to .nous");

        // Reread the .nous file.
        let mut json_2 = vec![];
        if let Ok(j) = fs::read(&nous_path.as_path()) {
            json_2 = j;
        }

        // Look for a discrepancy
        assert_eq!(json_1, json_2);

        // Clean up.
        env::set_current_dir(&old_dir).unwrap();
        // fs::remove_dir_all(&test_dir);
    }
}
