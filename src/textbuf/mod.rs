pub(crate) mod alphabet;
mod model;
mod raw_span;
pub(crate) mod regexes;
mod span;

pub use model::Model;
pub use span::Span;

#[cfg(test)]
mod tests {
    use super::alphabet::Alphabet;
    use super::model::Model;

    #[test]
    fn match_reverse() {
        let buffer = "lorem 127.0.0.1 lorem 255.255.255.255 lorem 127.0.0.1 lorem";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 3);
        assert_eq!(spans.first().unwrap().hint, "a");
        assert_eq!(spans.last().unwrap().hint, "c");
    }

    #[test]
    fn match_unique() {
        let buffer = "lorem 127.0.0.1 lorem 255.255.255.255 lorem 127.0.0.1 lorem";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = true;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 3);
        assert_eq!(spans.first().unwrap().hint, "a");
        assert_eq!(spans.last().unwrap().hint, "a");
    }

    #[test]
    fn match_docker() {
        let buffer = "latest sha256:30557a29d5abc51e5f1d5b472e79b7e296f595abcf19fe6b9199dbbc809c6ff4 20 hours ago";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 1);
        assert_eq!(
            spans.get(0).unwrap().text,
            "30557a29d5abc51e5f1d5b472e79b7e296f595abcf19fe6b9199dbbc809c6ff4"
        );
    }

    #[test]
    fn match_ansi_colors() {
        let buffer =
        "path: [32m/var/log/nginx.log[m\npath: [32mtest/log/nginx-2.log:32[mfolder/.nginx@4df2.log";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = true;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 3);
        assert_eq!(spans.get(0).unwrap().text, "/var/log/nginx.log");
        assert_eq!(spans.get(1).unwrap().text, "test/log/nginx-2.log");
        assert_eq!(spans.get(2).unwrap().text, "folder/.nginx@4df2.log");
    }

    #[test]
    fn match_paths() {
        let buffer =
        "Lorem /tmp/foo/bar_lol, lorem\n Lorem /var/log/boot-strap.log lorem ../log/kern.log lorem";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 3);
        assert_eq!(spans.get(0).unwrap().text, "/tmp/foo/bar_lol");
        assert_eq!(spans.get(1).unwrap().text, "/var/log/boot-strap.log");
        assert_eq!(spans.get(2).unwrap().text, "../log/kern.log");
    }

    #[test]
    fn match_home() {
        let buffer = "Lorem ~/.gnu/.config.txt, lorem";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 1);
        assert_eq!(spans.get(0).unwrap().text, "~/.gnu/.config.txt");
    }

    #[test]
    fn match_uuids() {
        let buffer = "Lorem ipsum 123e4567-e89b-12d3-a456-426655440000 lorem\n Lorem lorem lorem";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn match_shas() {
        let buffer = "Lorem fd70b5695 5246ddf f924213 lorem\n Lorem 973113963b491874ab2e372ee60d4b4cb75f717c lorem";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 4);
        assert_eq!(spans.get(0).unwrap().text, "fd70b5695");
        assert_eq!(spans.get(1).unwrap().text, "5246ddf");
        assert_eq!(spans.get(2).unwrap().text, "f924213");
        assert_eq!(
            spans.get(3).unwrap().text,
            "973113963b491874ab2e372ee60d4b4cb75f717c"
        );
    }

    #[test]
    fn match_ipv4s() {
        let buffer = "Lorem ipsum 127.0.0.1 lorem\n Lorem 255.255.10.255 lorem 127.0.0.1 lorem";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 3);
        assert_eq!(spans.get(0).unwrap().pattern, "ipv4");
        assert_eq!(spans.get(0).unwrap().text, "127.0.0.1");
        assert_eq!(spans.get(1).unwrap().pattern, "ipv4");
        assert_eq!(spans.get(1).unwrap().text, "255.255.10.255");
        assert_eq!(spans.get(2).unwrap().pattern, "ipv4");
        assert_eq!(spans.get(2).unwrap().text, "127.0.0.1");
    }

    #[test]
    fn match_ipv6s() {
        let buffer = "Lorem ipsum fe80::2:202:fe4 lorem\n Lorem 2001:67c:670:202:7ba8:5e41:1591:d723 lorem fe80::2:1 lorem ipsum fe80:22:312:fe::1%eth0";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 4);
        assert_eq!(spans.get(0).unwrap().text, "fe80::2:202:fe4");
        assert_eq!(
            spans.get(1).unwrap().text,
            "2001:67c:670:202:7ba8:5e41:1591:d723"
        );
        assert_eq!(spans.get(2).unwrap().text, "fe80::2:1");
        assert_eq!(spans.get(3).unwrap().text, "fe80:22:312:fe::1%eth0");
    }

    #[test]
    fn match_markdown_urls() {
        let buffer =
            "Lorem ipsum [link](https://github.io?foo=bar) ![](http://cdn.com/img.jpg) lorem";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 2);
        assert_eq!(spans.get(0).unwrap().pattern, "markdown-url");
        assert_eq!(spans.get(0).unwrap().text, "https://github.io?foo=bar");
        assert_eq!(spans.get(1).unwrap().pattern, "markdown-url");
        assert_eq!(spans.get(1).unwrap().text, "http://cdn.com/img.jpg");
    }

    #[test]
    fn match_urls() {
        let buffer = "Lorem ipsum https://www.rust-lang.org/tools lorem\n Lorem ipsumhttps://crates.io lorem https://github.io?foo=bar lorem ssh://github.io";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 4);
        assert_eq!(
            spans.get(0).unwrap().text,
            "https://www.rust-lang.org/tools"
        );
        assert_eq!(spans.get(0).unwrap().pattern, "url");
        assert_eq!(spans.get(1).unwrap().text, "https://crates.io");
        assert_eq!(spans.get(1).unwrap().pattern, "url");
        assert_eq!(spans.get(2).unwrap().text, "https://github.io?foo=bar");
        assert_eq!(spans.get(2).unwrap().pattern, "url");
        assert_eq!(spans.get(3).unwrap().text, "ssh://github.io");
        assert_eq!(spans.get(3).unwrap().pattern, "url");
    }

    #[test]
    fn match_emails() {
        let buffer =
            "Lorem ipsum <first.last+social@example.com> john@server.department.company.com lorem";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 2);
        assert_eq!(spans.get(0).unwrap().pattern, "email");
        assert_eq!(spans.get(0).unwrap().text, "first.last+social@example.com");
        assert_eq!(spans.get(1).unwrap().pattern, "email");
        assert_eq!(
            spans.get(1).unwrap().text,
            "john@server.department.company.com"
        );
    }

    #[test]
    fn match_pointer_addresses() {
        let buffer = "Lorem 0xfd70b5695 0x5246ddf lorem\n Lorem 0x973113tlorem";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 3);
        assert_eq!(spans.get(0).unwrap().pattern, "pointer-address");
        assert_eq!(spans.get(0).unwrap().text, "0xfd70b5695");
        assert_eq!(spans.get(1).unwrap().pattern, "pointer-address");
        assert_eq!(spans.get(1).unwrap().text, "0x5246ddf");
        assert_eq!(spans.get(2).unwrap().pattern, "pointer-address");
        assert_eq!(spans.get(2).unwrap().text, "0x973113");
    }

    #[test]
    fn match_hex_colors() {
        let buffer = "Lorem #fd7b56 lorem #FF00FF\n Lorem #00fF05 lorem #abcd00 lorem #afRR00";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 4);
        assert_eq!(spans.get(0).unwrap().text, "#fd7b56");
        assert_eq!(spans.get(1).unwrap().text, "#FF00FF");
        assert_eq!(spans.get(2).unwrap().text, "#00fF05");
        assert_eq!(spans.get(3).unwrap().text, "#abcd00");
    }

    #[test]
    fn match_ipfs() {
        let buffer = "Lorem QmRdbNSxDJBXmssAc9fvTtux4duptMvfSGiGuq6yHAQVKQ lorem Qmfoobar";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 1);
        assert_eq!(
            spans.get(0).unwrap().text,
            "QmRdbNSxDJBXmssAc9fvTtux4duptMvfSGiGuq6yHAQVKQ"
        );
    }

    #[test]
    fn match_process_port() {
        let buffer = "Lorem 5695 52463 lorem\n Lorem 973113 lorem 99999 lorem 8888 lorem\n   23456 lorem 5432 lorem 23444";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 8);
    }

    #[test]
    fn match_diff_a() {
        let buffer = "Lorem lorem\n--- a/src/main.rs";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 1);
        assert_eq!(spans.get(0).unwrap().pattern, "diff-a");
        assert_eq!(spans.get(0).unwrap().text, "src/main.rs");
    }

    #[test]
    fn match_diff_b() {
        let buffer = "Lorem lorem\n+++ b/src/main.rs";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 1);
        assert_eq!(spans.get(0).unwrap().pattern, "diff-b");
        assert_eq!(spans.get(0).unwrap().text, "src/main.rs");
    }

    #[test]
    fn match_datetime() {
        let buffer = "12 days ago = 2021-03-04T12:23:34 text";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 1);
        assert_eq!(spans.get(0).unwrap().pattern, "datetime");
        assert_eq!(spans.get(0).unwrap().text, "2021-03-04T12:23:34");
    }

    #[test]
    fn priority_between_regexes() {
        let buffer = "Lorem [link](http://foo.bar) ipsum CUSTOM-52463 lorem ISSUE-123 lorem\nLorem /var/fd70b569/9999.log 52463 lorem\n Lorem 973113 lorem 123e4567-e89b-12d3-a456-426655440000 lorem 8888 lorem\n  https://crates.io/23456/fd70b569 lorem";
        let lines = buffer.split('\n').collect::<Vec<_>>();
        let use_all_patterns = true;
        let named_pat = vec![];
        let custom: Vec<String> = ["(CUSTOM-[0-9]{4,})", "(ISSUE-[0-9]{3})"]
            .iter()
            .map(|&s| s.to_string())
            .collect();
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 9);
        assert_eq!(spans.get(0).unwrap().text, "http://foo.bar");
        assert_eq!(spans.get(1).unwrap().text, "CUSTOM-52463");
        assert_eq!(spans.get(2).unwrap().text, "ISSUE-123");
        assert_eq!(spans.get(3).unwrap().text, "/var/fd70b569/9999.log");
        assert_eq!(spans.get(4).unwrap().text, "52463");
        assert_eq!(spans.get(5).unwrap().text, "973113");
        assert_eq!(
            spans.get(6).unwrap().text,
            "123e4567-e89b-12d3-a456-426655440000"
        );
        assert_eq!(spans.get(7).unwrap().text, "8888");
        assert_eq!(
            spans.get(8).unwrap().text,
            "https://crates.io/23456/fd70b569"
        );
    }

    #[test]
    fn named_patterns() {
        let buffer = "Lorem [link](http://foo.bar) ipsum CUSTOM-52463 lorem ISSUE-123 lorem\nLorem /var/fd70b569/9999.log 52463 lorem\n Lorem 973113 lorem 123e4567-e89b-12d3-a456-426655440000 lorem 8888 lorem\n  https://crates.io/23456/fd70b569 lorem";
        let lines = buffer.split('\n').collect::<Vec<_>>();

        let use_all_patterns = false;
        use crate::textbuf::regexes::parse_pattern_name;
        let named_pat = vec![parse_pattern_name("url").unwrap()];

        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let spans = Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom,
            reverse,
            unique_hint,
        )
        .spans;

        assert_eq!(spans.len(), 2);
        assert_eq!(spans.get(0).unwrap().text, "http://foo.bar");
        assert_eq!(
            spans.get(1).unwrap().text,
            "https://crates.io/23456/fd70b569"
        );
    }
}
