#![no_main]
use libfuzzer_sys::fuzz_target;
use goblin_experimental as goblin;

fuzz_target!(|data: &[u8]| {
    let _ = goblin::Object::parse(data);
});
