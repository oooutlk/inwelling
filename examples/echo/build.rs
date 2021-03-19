use inwelling::*;

use std::{env, fs, path::PathBuf};

fn main() {
    let metadata_from_downstream = inwelling( Opts::default() )
        .sections
        .into_iter()
        .fold( String::new(), |acc, section|
            format!( "{}{:?} <{}>: {}\n"
                , acc
                , section.manifest
                , section.pkg
                , section.metadata.to_string() ));

    let out_path = PathBuf::from( env::var( "OUT_DIR" )
        .expect( "$OUT_DIR should exist." )
    ).join( "metadata_from_downstream" );

    fs::write(
        out_path,
        metadata_from_downstream
    ).expect( "metadata_from_downstream generated." );
}
