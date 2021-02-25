// Copyright 2018 oooutlk@outlook.com. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Problem To Solve
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
//! The information in section `[package.metadata.inwelling.{common ancestor}.*]`
//! in downstream crates' Cargo.toml files will be collected by `inwelling()`.
//!
//! # Examples
//!
//! See this [demo](https://github.com/oooutlk/inwelling/tree/main/examples/)
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
//! # Optional Metadata
//!
//! Cargo features can control whether to send metadata or not. in section
//! `[package.metadata.inwelling-{common ancestor}]`, a value of `feature = blah`
//! means that the metadata will be collected by inwelling if and only if blah
//! feature is enabled. See beta crate in examples for more.
//!
//! # Caveat
//!
//! ## Reverse Dependency
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
//!
//! ## Lacking Of `PWD` Environment Variable On Windows
//!
//! Without official support from cargo, this library requires environment variable
//! such as `PWD` to locate topmost crate's Cargo.toml. Unfortunately `PWD` is
//! missing on Windows platform. This library will panic if it is feeling no luck to
//! locate Cargo.toml. However, `PWD` is not mandatory, unless `inwelling()` told
//! you so.

use cargo_metadata::{
    CargoOpt,
    MetadataCommand,
};

use pals::Pid;

use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
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

    let mut manifest_path = None;
    let mut target_dir_defined_in_cmdline = false;

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
                        "--manifest-path" if cfg!( unix ) => if let Some( path ) = argv.next() {
                            manifest_path = Some( PathBuf::from( path ));
                        }
                        "--target-dir" => target_dir_defined_in_cmdline = true,
                        _ => (),
                    }
                }
            }
        }
    }

    let manifest_path = manifest_path.unwrap_or_else( ||
        if let Ok( cwd ) = env::var("PWD") {
            return PathBuf::from( cwd ).join( "Cargo.toml" );
        } else {
            if !target_dir_defined_in_cmdline {
                if let Ok( out_dir ) = env::var("OUT_DIR") {
                    let out_dir = Path::new( &out_dir );
                    let ancestors = out_dir.ancestors();
                    if let Some( manifest_dir ) = ancestors.skip(5).next() {
                        return manifest_dir.join( "Cargo.toml" );
                    }
                }
            }
            panic!("Failed to locate manifest path. Consider providing PWD environment variable.")
        }
    );

    if !manifest_path.exists() {
        panic!( "{:?} should be manifest file", manifest_path );
    }

    let metadata = command
        .manifest_path( &manifest_path )
        .exec()
        .expect("cargo metadata command should be executed successfully.");

    let build_name = env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME");

    let enabled_features = metadata.resolve.expect("package dependencies resolved.").nodes.iter().fold( HashMap::new(), |mut map, node| {
        map.insert( node.id.clone(), node.features.clone() );
        map
    });

    metadata.packages.into_iter().fold( Inwelling::default(), |mut inwelling, mut pkg| {
        let pkg_id = pkg.id.clone();

        let enabled = pkg.metadata
            .get( &format!( "inwelling-{}", &build_name ))
            .and_then( |section| section.get( "feature" ))
            .and_then( |feature| {
                let feature = feature.as_str().expect("feature name should be str.");
                Some( enabled_features[ &pkg_id ].iter().find( |&enabled_feature| enabled_feature == &feature ).is_some() )
            })
            .unwrap_or( true );

        if enabled {
            if let Some( section ) = pkg.metadata.get_mut("inwelling") {
                if let Some( metadata ) = section.get_mut( &build_name ) {
                    inwelling.sections.push( Section{
                        pkg      : pkg.name,
                        manifest : pkg.manifest_path,
                        metadata : metadata.take(),
                    });
                }
            }
        }

        inwelling
    })
}
