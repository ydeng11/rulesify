#[cfg(test)]
mod tests {
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn test_version_flag_prints_package_version() {
        let err = match Cli::try_parse_from(["rulesify", "--version"]) {
            Err(e) => e,
            Ok(_) => panic!("expected --version to exit with DisplayVersion"),
        };

        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
        assert_eq!(err.exit_code(), 0);
        assert!(err.to_string().contains(env!("CARGO_PKG_VERSION")));
    }
}
