#![no_std]

use mlua::prelude::*;
use alloc::string::{ToString, String};
use alloc::format;
use alloc::vec::Vec;

extern crate alloc;
// Module declarations

mod stringify;
mod boot;
mod eval;
mod pretty;
mod default;
mod assignment;
mod utils;