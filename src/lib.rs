// Copyright 2018 oooutlk@outlook.com. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Problem To Resolve
//!
//! Sometimes a crate needs to gather information from its downstream users.
//!
//! Frequently used mechanisms:
//!
//! - Cargo Features.
//!
//!   The are friendly to cargo tools but not applicable for passing free contents
//!   because they are predefined options.
//!
//! - Environment Variables.
//!
//!   They can pass free contents, but are not friendly to cargo tools.
//!
//! # Project Goal
//!
//! To provide a mechanism that is both friendly to cargo tools and able to pass
//! free contents.
//!
//! # Library Overview
//!
//! This library helps to send metadata through the hierarchy of crates, from
//! downstream crates to one of their common ancestor.
//!
//! The main API is `inwelling()`, which is expected to be called in `build.rs` of
//! the common ancestor crate.
//!
//! ```text
//! .      +--------------> [topmost crate]
//! .      |      3            |       ^
//! .      |                  4|       |8
//! .      |                   |       |
//! .      |                 [dependencies]
//! .      |2                  |       |
//! .      |                   |       |
//! .      |        (metadata) |5     7| (API)
//! .      |                   |       |
//! .      |        1          v   6   |
//! . inwelling() <---- build.rs ----> bindings.rs
//! .[inwelling crate]     [common ancestor]
//! ```
//!
//! The information in section `[packages.metadata.inwelling.{common ancestor}.*]`
//! in downstream crates' Cargo.toml files will be collected by `inwelling()`.
//!
//! # Examples
//!
//! See this [demo](https://github.com/oooutlk/inwelling/tree/main/src/examples/)
//! for more.
//!
//! The `echo` crate has build-dependency of inwelling crate:
//!
//! ```toml
//! [build-dependencies]
//! inwelling = { path = "../.." }
//! ```
//!
//! And provides `echo()` which simply returns what it recieves as strings.
//!
//! In `build.rs`:
//!
//! ```rust,no_run
//! use std::{env, fs, path::PathBuf};
//!
//! fn main() {
//!     let metadata_from_downstream = inwelling::inwelling()
//!         .sections
//!         .into_iter()
//!         .fold( String::new(), |acc, section|
//!             format!( "{}{:?} <{}>: {}\n"
//!                 , acc
//!                 , section.manifest
//!                 , section.pkg
//!                 , section.metadata.to_string() ));
//!
//!     let out_path = PathBuf::from( env::var( "OUT_DIR" )
//!         .expect( "$OUT_DIR should exist." )
//!     ).join( "metadata_from_downstream" );
//!
//!     fs::write(
//!         out_path,
//!         metadata_from_downstream
//!     ).expect( "metadata_from_downstream generated." );
//! }
//! ```
//!
//! In `lib.rs`:
//!
//! ```rust,no_run
//! pub fn echo() -> String {
//!     include_str!( concat!( env!( "OUT_DIR" ), "/metadata_from_downstream" ))
//!         .to_owned()
//! }
//! ```
//!
//! The gamma crate depends on alpha crate and conditionally depends on beta crate.
//! The beta crate depends on alpha crate. The alpha, beta and gamma ccrates all
//! depend on echo crate.
//!
//! ```text
//! .      +---------------> [gamma crate]    gamma=true
//! .      |                   .       ^           ^
//! .      |       gamma=true  .       |           |
//! .      |                   .       |           |
//! .      |            [beta crate]   |       beta=true
//! .      |                   |       |           |
//! .      |        beta=true  |       |           |
//! .      |                   |       |           |
//! .      |                 [alpha crate]    alpha=true
//! .      |                   |       |           |
//! .      |       alpha=true  |       |           |
//! .      |                   v       |           |
//! . inwelling() <---- build.rs ----> `echo()`----+
//! .[inwelling crate]       [echo crate]
//! ```
//!
//! In alpha crate's test code:
//!
//! ```rust,no_run
//! pub fn test() {
//!     let metadata = echo::echo();
//!     assert!( metadata.find("<alpha>: {\"alpha\":true}\n").is_some() );
//! }
//! ```
//!
//! # Caveat
//!
//! Collecting metadata from downstream and utilizing it in build process makes a
//! crate depending on its downstream crates. Unfortunately this kind of
//! reverse-dependency is not known to cargo. As a result, the changing of metadata
//! caused by modification of Cargo.toml files or changing of feature set will not
//! cause recompilation of the crate collecting metadata, which it should.
//!
//! To address this issue, simply do `cargo clean`, or more precisely,
//! `cargo clean --package {crate-collecting-metadata}` before running
//! `cargo build`. Substitute `{crate-collecting-metadata}` with actual crate name,
//! e.g. `cargo clean --package echo` in the examples above.

use cargo_metadata::{
    CargoOpt,
    MetadataCommand,
};

use pals::Pid;

use std::{
    env,
    path::PathBuf,
    process,
};

/// Metadata collected from downstream crates.
#[derive( Debug )]
pub struct Inwelling {
    /// sections gathered from downstream Cargo.toml files
    pub sections : Vec<Section>,
}

impl Default for Inwelling {
    fn default() -> Self {
        Inwelling{ sections: Vec::new() }
    }
}

/// Metadata collected from downstream crates, in `[package.metadata.inwelling.*]` sections.
#[derive( Debug )]
pub struct Section {
    /// name of the package which collects metadata from its downstream crates.
    pub pkg      : String,
    /// path of Cargo.toml.
    pub manifest : PathBuf,
    /// metadata represented in JSON.
    pub metadata : serde_json::value::Value,
}

/// Collects metadata from `[package.metadata.inwelling.*]` sections in downstream crates' Cargo.toml files.
pub fn inwelling() -> Inwelling {
    let mut command = MetadataCommand::new();
    let mut manifest_path = PathBuf::from( env::var("PWD").unwrap() ).join( "Cargo.toml" );

    let pals = pals::pals();
    if let Ok( pals ) = pals {
        if let Some( parent ) = pals.parent_of( Pid( process::id() )) {
            if let Some( parent ) = pals.parent_of( parent.ppid ) {
                let mut argv = parent.argv();
                while let Some( arg ) = argv.next() {
                    match arg {
                        "--all-features" => {
                            command.features( CargoOpt::AllFeatures );
                        },
                        "--features" => if let Some( features ) = argv.next() {
                            command.features( CargoOpt::SomeFeatures( features
                                .split_ascii_whitespace()
                                .map( ToOwned::to_owned )
                                .collect()
                            ));
                        },
                        "--no-default-features" => {
                            command.features( CargoOpt::NoDefaultFeatures );
                        },
                        "--manifest-path" => if let Some( path ) = argv.next() {
                            manifest_path = PathBuf::from( path );
                        },
                        _ => (),
                    }
                }
            }
        }
    }

    let metadata = command
        .manifest_path( &manifest_path )
        .exec()
        .unwrap();

    let build_name = env::var("CARGO_PKG_NAME").unwrap();

    metadata.packages.into_iter().fold( Inwelling::default(), |mut inwelling, mut pkg| {
        if let Some( section ) = pkg.metadata.get_mut("inwelling") {
            if let Some( metadata ) = section.get_mut( &build_name ) {
                inwelling.sections.push( Section{
                    pkg      : pkg.name,
                    manifest : pkg.manifest_path,
                    metadata : metadata.take(),
                });
            }
        }
        inwelling
    })
}
