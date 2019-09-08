extern crate url;
extern crate git2;

use std::str::FromStr;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use url::Url;
use git2::Repository;

pub trait Fetchable {
    fn fetch(&self, dest: &Address) -> Result<(), &'static str>;
}


pub struct Project {
    origin: Address,
    resources: Vec<Resource>,
}


pub struct Resource {
    origin: Address,
    kind: ResourceKind,
    name: String,
}


pub enum ResourceKind {
    Analysis,
    Dataset,
    GitRepository,
}


pub enum Address {
    Local(PathBuf),
    Remote(Url),
}


impl Project {
    pub fn new(origin: Address) -> Self {
        Project {
            origin,
            resources: vec![],
        }
    }
}


impl Fetchable for Project {
    /// Retrieve the .nous file and all contained resources
    fn fetch(&self, dest: &Address) -> Result<(), &'static str> {
        // First, retrieve the .nous file.

        // Next, retrieve all the associated resources described therein.
        Ok(())
    }
}


impl Resource {
    pub fn new(origin: Address, kind: ResourceKind) -> Self {
        Resource {
            origin,
            kind,
            name: String::new(),
        }
    }
}


impl Fetchable for Resource {
    /// Retrieve the resource and put into destination
    fn fetch(&self, dest: &Address) -> Result<(), &'static str> {
        match &self.origin {

            Address::Local(orig_path) => {
                dbg!("Acquiring a local resource");
                match dest {

                    Address::Local(dest_path) => {
                        if orig_path.exists() {
                            if cfg!(unix) {
                                dbg!("Symlinking resource!");
                                // TODO: handle this error
                                match symlink(orig_path, dest_path) {
                                    Ok(_) => Ok(()),
                                    Err(_) => Err("Failed to symlink."),
                                }
                            } else {
                                unimplemented!("This platform not supported.");
                                Ok(())
                            }
                            //Ok(())
                        } else {
                            Err("Origin does not exist.")
                        }
                    },

                    Address::Remote(dest_path) => {
                        Ok(())
                    }

                }

            },

            Address::Remote(url) => {
                dbg!("Acquiring a remote resource");
                match dest {

                    Address::Local(dest_path) => {
                        // If it's a git repo
                        let dest_path: &str = dest_path.to_str().unwrap();
                        dbg!("Attempting to clone repository.");
                        match Repository::clone(url.as_str(), dest_path) {
                            Ok(_) => Ok(()),
                            Err(e) => Err("Failed to clone repository.")
                        }

                    },

                    Address::Remote(dest_path) => {
                        Ok(())
                    },

                }
            },

        }
    }


}


impl Address {
    pub fn new(uri: String) -> Self {
        // Figure out if local or remote as stupidly as possible
        let uri_wrap = Url::from_str(&uri);
        match uri_wrap {
            Ok(url) => Address::Remote(url),
            Err(_) => {
                // Assume that it must be a local resource
                Address::Local(PathBuf::from(&uri))
            },
        }
    }
}
