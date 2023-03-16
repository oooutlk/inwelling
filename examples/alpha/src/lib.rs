pub fn test() {
    let metadata = echo::echo();
    assert!( metadata.find("alpha = \"the first letter\"").is_some() );
}

#[cfg( test )]
mod tests {
    #[test]
    fn it_works() {
        super::test();
    }
}
