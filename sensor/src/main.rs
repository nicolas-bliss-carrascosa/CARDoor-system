// Origional template taken from https://github.com/flipperzero-rs/flipperzero-template/tree/main 
#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_alloc;
extern crate flipperzero_rt;
extern crate alloc;

use core::ffi::CStr;
use flipperzero::println;
use flipperzero_rt::{entry, manifest};

// Define the FAP Manifest
manifest!(
    name = "Card Sensor",
    app_version = 1,
    has_icon = true,
    icon = "rustacean-10x10.icon",
);

// Define the entry function
entry!(main);

// Entry point
fn main(_args: Option<&CStr>) -> i32 {
    println!("Hello, Rust!");
    flipperzero::dialogs::alert("Text");
    0
}