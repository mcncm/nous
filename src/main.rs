use nous::*;

fn main() {
    let loc_origin = Address::new(String::from("/home/mcncm/proj/rust/nous/ex_origin.txt"));
    let loc_target = Address::new(String::from("/home/mcncm/proj/rust/nous/ex_target.txt"));

    let rem_origin = Address::new(String::from("https://github.com/mcncm/interpy"));
    let rem_target = Address::new(String::from("/home/mcncm/proj/rust/nous/rem_target"));

    let loc_res = Resource::new(loc_origin, ResourceKind::Dataset);
    let rem_res = Resource::new(rem_origin, ResourceKind::Dataset);

    loc_res.fetch(&loc_target);
    rem_res.fetch(&rem_target);
}
