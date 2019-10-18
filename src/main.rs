use nous::*;
use std::io;
use std::env;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let loc_origin = Address::new(String::from("/home/mcncm/proj/rust/nous/ex_origin.txt"));
    let rem_origin = Address::new(String::from("https://github.com/mcncm/interpy"));


    let interpy_repo = GitRepository{origin: rem_origin,
                                     dest_dir: env::current_dir()?,
                                     name: String::from("interpy"),
    };

    interpy_repo.fetch();
    Ok(())

    /*
    let img_origin = Address::new(String::from("https://upload.wikimedia.org/wikipedia/commons/f/f4/Ringedteal.PNG"));
    let img_target = Address::new(String::from("/home/mcncm/proj/rust/nous/duck.png"));

    let loc_res = Resource::new(loc_origin, ResourceKind::File, String::from("a_file.txt"));
    let rem_res = Resource::new(rem_origin, ResourceKind::GitRepository, String::from("my-repo"));
    let img_res = Resource::new(img_origin, ResourceKind::File, String::from("duck.png"));

    let mut proj = Project::new(Address::Local(std::path::PathBuf::from("/home/mcncm/tmp")));
    proj.push(&loc_res);
    proj.push(&rem_res);
    proj.push(&img_res);

    //let mut proj = project!("/home/mcncm/tmp", &loc_res, &rem_res, &img_res);

    proj.fetch(&Address::new(String::from("/home/mcncm/proj/rust/nous/my-proj")))
        .expect("Problem fetching project");


    //loc_res.fetch(&loc_target);
    //rem_res.fetch(&rem_target);
    */
}
