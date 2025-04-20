#![no_std]

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use mlua::prelude::*;

extern crate alloc;
// Module declarations

mod assignment;
mod boot;
mod default;
mod eval;
mod pretty;
mod stringify;
mod utils;
