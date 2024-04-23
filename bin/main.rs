use wit2wadm_lib::convert::wit2wadm;
use wit_parser::Resolve;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let mut resolve = Resolve::new();
    // TODO: Depending on args we could parse a component

    let wit_folder = args.get(1).expect("missing wit folder");
    let world_name = args.get(2).expect("missing world name");

    resolve
        .push_path(wit_folder)
        .expect("should be able to load wits");

    wit2wadm(resolve, world_name)
}
