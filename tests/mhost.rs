extern crate assert_cli;

#[cfg(test)]
mod integration {
    use assert_cli;

    #[test]
    fn mhost_wo_args() {
        mhost()
            .fails()
            .and()
            .stderr().contains("error: The following required arguments were not provided:")
            .unwrap()
    }

    #[test]
    fn mhost_simple_lookup() {
        mhost()
            .with_args(&["www.example.com"])
            .succeeds()
            .and()
            .stdout().contains("* IPv4: 93.184.216.34")
            .and()
            .stdout().contains("* IPv6: 2606:2800:220:1:248:1893:25c8:1946")
            .unwrap()
    }

    #[test]
    fn mhost_simple_ipv4_reverse_lookup() {
        mhost()
            .with_args(&["8.8.8.8"])
            .succeeds()
            .and()
            .stdout().contains("* PTR: google-public-dns-a.google.com.")
            .unwrap()
    }

    #[test]
    fn mhost_simple_ipv6_reverse_lookup() {
        mhost()
            .with_args(&["2001:4860:4860::8888"])
            .succeeds()
            .and()
            .stdout().contains("* PTR: google-public-dns-a.google.com.")
            .unwrap()
    }

    fn mhost() -> assert_cli::Assert {
        assert_cli::Assert::command(&["cargo", "run", "--features=bin"])
    }
}