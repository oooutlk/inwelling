pub fn test() {
    let metadata = echo::echo();
    assert!( metadata.find("beta = \"the second letter\"").is_none() );
}

#[cfg( test )]
mod tests {
    #[test]
    fn it_works() {
        alpha::test();
        super::test();
    }
}
