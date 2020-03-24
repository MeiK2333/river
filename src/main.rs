extern crate libc;

mod runner;

fn main() {
    process();
}

fn process() {
    let pid;
    unsafe {
        pid = libc::fork();
    }
    if pid == 0 {
        println!("children!");
    } else {
        println!("{:?}", pid);
    }
}
