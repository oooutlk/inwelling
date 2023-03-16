pub fn test() {
    let metadata = echo::echo();
    dbg!( &metadata ); 
    assert!( metadata.find("gamma = \"the third letter\"").is_some() );
}

#[cfg( test )]
mod tests {
    #[test]
    fn it_works() {
        alpha::test();
        beta::test();
        super::test();
    }
}
