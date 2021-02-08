pub fn test() {
    let metadata = echo::echo();
    assert!( metadata.find("<alpha>: {\"alpha\":true}\n").is_some() );
}

#[cfg( test )]
mod tests {
    #[test]
    fn it_works() {
        println!( "{}", echo::echo() );

        super::test();
    }
}
