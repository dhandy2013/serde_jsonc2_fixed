//! Test deserializing JSONC with many consecutive comments.
//! On earlier versions of serde_jsonc2 these tests would not just fail
//! but get a fatal error due to stack overflow.

use serde_jsonc2::Value;
use std::{io::Write, thread};

const STACK_SIZE: usize = 2048;

/// Number of consecutive comments to add to the JSONC file.
/// Without the stack overflow fix, on my machine with a 2k thread stack size,
/// I can have a maximum of
/// - 66 consecutive line comments
/// - 58 consecutive block comments
/// before I get a stack overflow error.
const NUM_CONSECUTIVE_COMMENTS: usize = 128;

#[test]
fn test_many_consecutive_block_comments() {
    let path = create_jsonc_bytes(|w, i| {
        // Create block comment
        writeln!(w, "/* Comment line {i} */").unwrap()
    });
    load_jsonc_bytes_in_thread(path);
}

#[test]
fn test_many_consecutive_line_comments() {
    let path = create_jsonc_bytes(|w, i| {
        // Create line comment
        writeln!(w, "// Comment line {i}").unwrap()
    });
    load_jsonc_bytes_in_thread(path);
}

fn create_jsonc_bytes<F>(mut create_comment_callback: F) -> Vec<u8>
where
    F: FnMut(&mut dyn Write, usize),
{
    let mut w = Vec::<u8>::new();
    for i in 0..NUM_CONSECUTIVE_COMMENTS {
        create_comment_callback(&mut w, i);
    }
    writeln!(w, "{{\"a\": 1}}").unwrap();
    w.flush().unwrap();
    w
}

fn load_jsonc_bytes_in_thread(bytes: Vec<u8>) {
    let join_handle = thread::Builder::new()
        .name(String::from("many_consecutive_comments"))
        .stack_size(STACK_SIZE)
        .spawn(move || serde_jsonc2::from_reader::<_, Value>(bytes.as_slice()))
        .unwrap();
    let thread_result = join_handle.join();
    match thread_result {
        Ok(run_result) => match run_result {
            Ok(_) => {
                // Many consecutive comments loaded Ok, test passes
                return;
            }
            Err(err) => {
                panic!("Error loading consecutive comments: {err}");
            }
        },
        Err(err) => {
            panic!("Panic in thread loading consecutive comments: {err:?}");
        }
    }
}
