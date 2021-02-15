pub fn test() {
    let metadata = echo::echo();
    assert!( metadata.find("\"gamma\":true").is_some() );
}

#[cfg( test )]
mod tests {
    #[test]
    fn it_works() {
        println!( "{}", echo::echo() );

        alpha::test();

        #[cfg( features = "beta" )]
        {
            beta::test();
        }

        super::test();
    }
}
