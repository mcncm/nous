use nous::*;
use std::path::PathBuf;

fn main() {
    let loc_origin = Address::new(String::from("/home/mcncm/proj/rust/nous/ex_origin.txt"));
    let loc_target = Address::new(String::from("/home/mcncm/proj/rust/nous/ex_target.txt"));

    let rem_origin = Address::new(String::from("https://github.com/mcncm/interpy"));
    let rem_target = Address::new(String::from("/home/mcncm/proj/rust/nous/rem_target"));

    let loc_res = Resource::new(loc_origin, ResourceKind::File, String::from("a_file.txt"));
    let rem_res = Resource::new(rem_origin, ResourceKind::GitRepository, String::from("my-repo"));

    let mut proj = Project::new(Address::Local(std::path::PathBuf::from("/home/mcncm/tmp")));
    proj.push(&loc_res);
    proj.push(&rem_res);

    proj.fetch(&Address::new(String::from("/home/mcncm/proj/rust/nous/my-proj"))).unwrap();


    //loc_res.fetch(&loc_target);
    //rem_res.fetch(&rem_target);
}
