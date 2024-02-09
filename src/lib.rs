// Copyright 2018 oooutlk@outlook.com. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Project Goal
//!
//! To provide a mechanism for upstream crates to collect information from
//! downstream crates.
//!
//! # Information collected from downstream crates
//!
//! Invoking `collect_downstream()` will collect the following information from
//! crates which called `inwelling::to()` in its `build.rs`.
//!
//! - Package name.
//!
//! - Metadata defined in `Cargo.toml`.
//!
//! - Manifest paths of `Cargo.toml`.
//!
//! - Source file paths(optional). Call `collect_downstream()` with the argument
//! `inwelling::Opt::dump_rs_paths == true` to collect.
//!
//! # Quickstart
//!
//! 1. The upstream crate e.g. `crate foo` calls `inwelling::collect_downstream()`
//!    in its `build.rs` and do whatever it want to generate APIs for downstream.
//!
//! 2. The downstream crate e.g. `crate bar` calls `inwelling::to()` in its
//!    `build.rs`.
//!
//!    ```rust,no_run
//!    // build.rs
//!    fn main() { inwelling::to( "foo" ); }
//!    ```
//!
//!    To send some metadata to upstream `crate foo`, encode them in `Cargo.toml`'s
//!    package metadata.
//!
//!    ```toml
//!    [package.metadata.inwelling.foo]
//!    answer = { type = "integer", value = "42" }
//!    ```

use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use walkdir::WalkDir;

/// Information collected from downstream crates.
#[derive( Debug )]
pub struct Downstream {
    pub packages : Vec<Package>,
}

impl Default for Downstream {
    fn default() -> Self {
        Downstream{ packages: Vec::new() }
    }
}

/// Information collected from one downstream crate. Including:
///
/// - Package name.
///
/// - Cargo.toml file' path.
///
/// - metadata from `[package.metadata.inwelling.*]` section in Cargo.toml file.
///
/// - Optional .rs file paths.
#[derive( Debug )]
pub struct Package {
    /// name of the package which called `inwelling::to()` in its `build.rs`.
    pub name     : String,
    /// path of `Cargo.toml`.
    pub manifest : PathBuf,
    /// metadata represented in Toml.
    pub metadata : toml::Value,
    /// .rs files under src/, examples/ and tests/ directories if `dump_rs_file`
    /// is true, otherwise `None`.
    pub rs_paths : Option<Vec<PathBuf>>,
}

fn scan_rs_paths( current_dir: impl AsRef<Path>, rs_paths: &mut Vec<PathBuf> ) {
    if let Ok( entries ) = current_dir.as_ref().read_dir() {
        for entry in entries {
            if let Ok( entry ) = entry {
                let path = entry.path();
                if path.is_dir() {
                    scan_rs_paths( path, rs_paths );
                } else if let Some( extention ) = path.extension() {
                    if extention == "rs" {
                        rs_paths.push( path );
                    }
                }
            }
        }
    }
}

/// Options passed to inwelling().
pub struct Opts {
    /// build.rs using inwelling() will re-run if downstream crates' Cargo.toml files have been changed.
    pub watch_manifest : bool,
    /// build.rs using inwelling() will re-run if downstream crates' .rs files have been changed.
    pub watch_rs_files : bool,
    /// if this flag is true, inwelling()'s returning value will contain .rs file paths.
    pub dump_rs_paths  : bool,
}

impl Default for Opts {
    fn default() -> Opts {
        Opts {
            watch_manifest : true,
            watch_rs_files : false,
            dump_rs_paths  : false,
        }
    }
}

/// Collects information from downstream crates. Including:
///
/// - Package names.
///
/// - Cargo.toml files' paths.
///
/// - metadata from `[package.metadata.inwelling.*]` sections in Cargo.toml files.
///
/// - Optional .rs file paths.
pub fn collect_downstream( Opts{ watch_manifest, watch_rs_files, dump_rs_paths }: Opts ) -> Downstream {
    let build_name = env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME");

    let manifest_paths = locate_manifest_paths();

    manifest_paths.into_iter().fold( Downstream::default(), |mut inwelling, (manifest_path, upstreams)| {
        if upstreams.contains( &build_name ) {
            let cargo_toml =
                fs::read_to_string( PathBuf::from( &manifest_path ))
                .expect( &format!( "to read {:?}", manifest_path ))
                .parse::<toml::Table>()
                .expect( &format!( "{:?} should be a valid manifest", manifest_path ));
            let package = cargo_toml.get( "package" )
                .expect( &format!( "{:?} should contain '[package]' section", manifest_path ));
            let package_name = package.as_table()
                .expect( &format!( "[package] section in {:?} should contain key-value pair(s)", manifest_path ))
                .get( "name" )
                .expect( &format!( "{:?} should contain package name", manifest_path ))
                .as_str()
                .expect( &format!( "{:?}'s package name should be a string", manifest_path ))
                .to_owned();

            let mut rs_paths = Vec::new();

            if watch_manifest {
                println!( "cargo:rerun-if-changed={}", manifest_path.to_str().unwrap() );
            }
            if dump_rs_paths || watch_rs_files {
                let manifest_dir = manifest_path.parent().unwrap();
                scan_rs_paths( &manifest_dir.join( "src"      ), &mut rs_paths );
                scan_rs_paths( &manifest_dir.join( "examples" ), &mut rs_paths );
                scan_rs_paths( &manifest_dir.join( "tests"    ), &mut rs_paths );
                if watch_rs_files {
                    rs_paths.iter().for_each( |rs_file|
                        println!( "cargo:rerun-if-changed={}", rs_file.to_str().unwrap() ));
                }
            }
            if let Some( metadata ) = package.get( "metadata" ) {
                if let Some( metadata_inwelling ) = metadata.get("inwelling") {
                    if let Some( metadata_inwelling_build ) = metadata_inwelling.get( &build_name ) {
                        inwelling.packages.push( Package{
                            name     : package_name,
                            manifest : manifest_path,
                            metadata : metadata_inwelling_build.clone(),
                            rs_paths : if dump_rs_paths { Some( rs_paths )} else { None },
                        });
                    }
                }
            }
        }

        inwelling
    })
}

// the path of the file that stores the downstream crate's manifest directory.
const MANIFEST_DIR_INWELLING: &'static str = "manifest_dir.inwelling";

fn wait_for_other_builds( build_dir: &Path ) {
    let mut generated = HashSet::<PathBuf>::new();
    let mut waiting = true;
    while waiting {
        thread::sleep( Duration::from_secs(5) );
        waiting = false;
        for entry in WalkDir::new( build_dir ) {
            let entry = entry.unwrap();
            let path = entry.path();
            if generated.insert( path.to_owned() ) {
                waiting = true;
            }
        }
    }
    eprintln!("{generated:#?}");
}

fn locate_manifest_paths() -> HashMap<PathBuf,Vec<String>> {
    let mut path_bufs = HashMap::new();

    let out_dir = PathBuf::from( env::var( "OUT_DIR" ).expect( "$OUT_DIR should exist." ));
    let ancestors = out_dir.ancestors();
    let build_dir = ancestors.skip(2).next().expect( "'build' directory should exist." );

    wait_for_other_builds( &build_dir );

    let mut pending = true;
    while pending {
        pending = false;
        for entry in build_dir.read_dir().expect( &format!( "to list all sub dirs in {:?}", build_dir )) {
            if let Ok( entry ) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let inwelling_file_path = path.join("out").join( MANIFEST_DIR_INWELLING );
                    if inwelling_file_path.exists() {
                        let contents = fs::read_to_string( &inwelling_file_path )
                            .expect( &format!( "to read {:?} to get one manifest path", inwelling_file_path ));
                        let mut lines = contents.lines();
                        let manifest_dir = lines.next()
                            .expect( &format!( "{:?} should contain the line of manifest dir.", inwelling_file_path ));
                        path_bufs
                            .entry( PathBuf::from( manifest_dir ).join( "Cargo.toml" ))
                            .or_insert_with( || lines.map( ToOwned::to_owned ).collect() );
    }}}}}
    path_bufs
}

/// Allow the upstream crate to collect information from this crate.
// The first line is manifest_dir
// The rest lines are upstream package names, one per line.
pub fn to( upstream: &str ) {
    let out_path =
        PathBuf::from(
            env::var( "OUT_DIR" )
                .expect( "$OUT_DIR should exist." )
        ).join( MANIFEST_DIR_INWELLING );
    if out_path.exists() {
        let mut f = File::options().append( true ).open( &out_path )
            .expect( &format!( "{:?} should be opened for appending.", out_path ));
        writeln!( &mut f, "{}", upstream )
            .expect( &format!( "An upstream name should be appended to {:?}.", out_path ));
    } else {
        let manifest_dir =
            env::var( "CARGO_MANIFEST_DIR" )
                .expect( "$CARGO_MANIFEST_DIR should exist." );
        fs::write(
            out_path,
            format!( "{}\n{}\n", manifest_dir, upstream )
        ).expect( "manifest_dir.txt generated." );
    }
}
