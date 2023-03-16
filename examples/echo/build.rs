use inwelling::*;

use std::{env, fs, path::PathBuf};

fn main() {
    let metadata_from_downstream = collect_downstream( Opts::default() )
        .packages
        .into_iter()
        .fold( String::new(), |acc, package|
            format!( "{}{:?} <{}>: {}\n"
                , acc
                , package.manifest
                , package.name
                , package.metadata.to_string() ));

    let out_path = PathBuf::from( env::var( "OUT_DIR" )
        .expect( "$OUT_DIR should exist." )
    ).join( "metadata_from_downstream" );

    fs::write(
        out_path,
        metadata_from_downstream
    ).expect( "metadata_from_downstream generated." );
}
