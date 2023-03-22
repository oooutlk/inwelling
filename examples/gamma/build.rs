fn main() {
    if std::env::var( "CARGO_FEATURE_TO_ECHO" ).is_ok() {
        inwelling::to( "echo" );
    }
}
