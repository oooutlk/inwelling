pub fn test() {
    let metadata = echo::echo();
    assert!( metadata.find("\"alpha\":true").is_some() );
}

#[cfg( test )]
mod tests {
    #[test]
    fn it_works() {
        println!( "{}", echo::echo() );

        super::test();
    }
}
