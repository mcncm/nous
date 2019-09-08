extern crate url;
extern crate git2;

use std::fs;
use std::str::FromStr;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use url::Url;
use git2::Repository;


pub trait Fetchable<E> {
    fn fetch(&self, dest: &Address) -> Result<(), E>;
}


pub struct Config {
    symlink_local_resources: bool,
}

#[derive(Clone)]
pub enum Address {
    Local(PathBuf),
    Remote(Url),
}


pub struct Project {
    origin: Address,
    resources: Vec<Resource>,
    name: String,
}


#[derive(Clone)]
pub struct Resource {
    origin: Address,
    kind: ResourceKind,
    name: String,
}


#[derive(Clone)]
pub enum ResourceKind {
    // Might be less unweildy wiht generics than with an enum.
    GitRepository,
    File,
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


impl Project {
    pub fn new(origin: Address) -> Self {
        Project {
            origin,
            resources: vec![],
            name: String::new(),
        }
    }

    pub fn push(&mut self, res: &Resource) {
        self.resources.push(res.clone())
    }

    fn fetch_local(&self, dest_path: &PathBuf) -> Result<(), Vec<&'static str>> {
        // Fetch a local project, assuming all the resources are present
        // TODO: acquire resource handles,  ...
        fs::create_dir(dest_path).unwrap();

        // Next, retrieve all the associated resources described therein.
        // This needs to be about 1000x better
        let mut errs: Vec<&'static str> = vec![];
        for res in self.resources.iter() {
            let res_dest_path = dest_path.join(&res.name);
            res.fetch(&Address::Local(res_dest_path)).unwrap_or_else(|err| {
                errs.push("Problem fetching resource");
            });
        }

        if errs.len() > 0 {
            Err(errs)
        } else {
            Ok(())
        }
    }
}


impl Fetchable<Vec<&'static str>> for Project {
    /// Retrieve the .nous file and all contained resources
    fn fetch(&self, dest: &Address) -> Result<(), Vec<&'static str>> {
        // First, retrieve the .nous file.

        // Create the destination directory if it doesn't exist
        match dest {
            Address::Local(dest_path) => {
                self.fetch_local(dest_path)?;
                Ok(())
            },
            _ => unimplemented!("Don't support remote project destinations yet")
        }

    }
}


impl Resource {
    pub fn new(origin: Address, kind: ResourceKind, name: String) -> Self {
        Resource {
            origin,
            kind,
            name,
        }
    }

    fn fetch_local_to_local(&self, orig_path: &PathBuf, dest_path: &PathBuf) -> Result<(), &'static str> {
        // TODO: should symlink, hardlink or copy depending on config setting
        // NOTE: same as note below.
        if orig_path.exists() {
            if cfg!(unix) {
                match symlink(orig_path, dest_path) {
                    Ok(_) => Ok(()),
                    Err(_) => Err("Failed to symlink."),
                }
            } else {
                unimplemented!("This platform not supported");
                //Ok(())
            }
        } else {
            Err("Origin does not exist.")
        }
    }

    fn fetch_remote_to_local(&self, orig_url: &Url, dest_path: &PathBuf) -> Result<(), &'static str> {
        // NOTE: It's a little ugly that this attempt at refactoring takes
        // `orig_url` as a parameter when this struct already has an `origin`
        // field. But consider a few things:
        //
        // 1) This is a private method, so it's not going to be mis-called by
        // anybody else.
        //
        // 2) It's possible that the `orig_url` is in some way a transformation
        // of the `origin` field. (On the other other hand, however, the origin
        // should probaby be sanitized at the interface--when it's first
        // received.)
        let dest_path: &str = dest_path.to_str().unwrap();
        dbg!("Attempting to clone repository.");
        match &self.kind {
            GitRepository => match Repository::clone(orig_url.as_str(), dest_path) {
                Ok(_) => Ok(()),
                Err(e) => Err("Failed to clone repository.")
            },
            _ => unimplemented!("Non-git remote origins not implemented")
        }
    }
}


impl Fetchable<&'static str> for Resource {
    /// Retrieve the resource and put into destination

    //TODO: this is a little bit unweildly, but I'm not quite sure how to
    // refactor it yet
    fn fetch(&self, dest: &Address) -> Result<(), &'static str> {
        match &self.origin {

            Address::Local(orig_path) => {
                dbg!("Acquiring a local resource");
                match dest {

                    Address::Local(dest_path) => {
                        self.fetch_local_to_local(orig_path, dest_path)
                    },

                    Address::Remote(dest_path) => {
                        unimplemented!("Remote destinations not implemented.")
                    }

                }

            },

            Address::Remote(orig_url) => {
                dbg!("Acquiring a remote resource");
                match dest {

                    Address::Local(dest_path) => {
                        self.fetch_remote_to_local(orig_url, dest_path)
                    },

                    Address::Remote(dest_path) => {
                        unimplemented!("Remote destinations not implemented.")
                    },
                }
            },
        }
    }
}
