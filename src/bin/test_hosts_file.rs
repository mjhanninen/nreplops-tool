use nreplops_tool::hosts_files::*;

fn main() {
    let hosts = load_default_hosts_files().unwrap();
    println!("{:#?}", hosts);
}
