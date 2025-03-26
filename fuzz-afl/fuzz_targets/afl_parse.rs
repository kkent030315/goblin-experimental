#[macro_use]
extern crate afl;
use goblin_experimental as goblin;

fn main() {
    fuzz!(|data: &[u8]| {
        let _ = goblin::Object::parse(data);
    });
}
