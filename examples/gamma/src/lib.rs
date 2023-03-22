pub fn test() {
    let metadata = echo::echo();
    assert_eq!( metadata.find("gamma = \"the third letter\"").is_some(), cfg!( feature = "to_echo" ));
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
