pub fn echo() -> String {
    include_str!( concat!( env!( "OUT_DIR" ), "/metadata_from_downstream" ))
        .to_owned()
}
