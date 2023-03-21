#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use kotka::Bif;

    #[test]
    fn test_bif() {
        let mut path = std::path::PathBuf::from_str("example_files/kotor2/").unwrap();
        let bif = Bif::new(&mut path, None, None).unwrap();

        assert_eq!(bif.path, path)
    }
}
