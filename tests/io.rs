extern crate rstest_reuse;
use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use rstest::rstest;
use rstest_reuse::{apply, template};
use std::ffi::OsString;
use test_binary::build_test_binary_once;

build_test_binary_once!(linelen, "tests/bins");
build_test_binary_once!(revbytes, "tests/bins");

enum PathPolicy {
    Default,
    Hyphen,
    Path,
}

struct IOPolicy {
    input: PathPolicy,
    output: PathPolicy,
}

impl IOPolicy {
    fn run<S: ?Sized + AsRef<[u8]>>(
        &self,
        cmdpath: OsString,
        input: &'static S,
        output: &'static S,
    ) {
        let input: &'static [u8] = input.as_ref();
        let output: &'static [u8] = output.as_ref();
        let mut cmd = Command::new(cmdpath);
        let tmpdir = TempDir::new().unwrap();
        let outfile = match self.output {
            PathPolicy::Default => None,
            PathPolicy::Hyphen => {
                cmd.arg("-o-");
                None
            }
            PathPolicy::Path => {
                let tmpfile = tmpdir.child("output.dat");
                cmd.arg("--outfile").arg(tmpfile.path());
                Some(tmpfile)
            }
        };
        let infile = match self.input {
            PathPolicy::Default => {
                cmd.write_stdin(input);
                None
            }
            PathPolicy::Hyphen => {
                cmd.arg("-").write_stdin(input);
                None
            }
            PathPolicy::Path => {
                let tmpfile = tmpdir.child("input.dat");
                tmpfile.write_binary(input).unwrap();
                cmd.arg(tmpfile.path());
                Some(tmpfile)
            }
        };
        let r = cmd.assert().success();
        if let Some(p) = infile {
            p.assert(input);
        }
        match outfile {
            Some(p) => {
                p.assert(output);
            }
            None => {
                r.stdout(predicate::eq(output));
            }
        }
    }
}

#[template]
#[rstest]
#[test]
#[case(IOPolicy {input: PathPolicy::Default, output: PathPolicy::Default})]
#[case(IOPolicy {input: PathPolicy::Default, output: PathPolicy::Hyphen})]
#[case(IOPolicy {input: PathPolicy::Default, output: PathPolicy::Path})]
#[case(IOPolicy {input: PathPolicy::Hyphen, output: PathPolicy::Default})]
#[case(IOPolicy {input: PathPolicy::Hyphen, output: PathPolicy::Hyphen})]
#[case(IOPolicy {input: PathPolicy::Hyphen, output: PathPolicy::Path})]
#[case(IOPolicy {input: PathPolicy::Path, output: PathPolicy::Default})]
#[case(IOPolicy {input: PathPolicy::Path, output: PathPolicy::Hyphen})]
#[case(IOPolicy {input: PathPolicy::Path, output: PathPolicy::Path})]
fn policies(#[case] policy: IOPolicy) {}

#[apply(policies)]
fn test_lines_and_create(#[case] policy: IOPolicy) {
    policy.run(
        path_to_linelen(),
        concat!(
            "1\n",
            "To\n",
            "Tre\n",
            "Four\n",
            "The longest line in the file\n",
            "\n",
            "Goodbye\n",
        ),
        "1\n2\n3\n4\n28\n0\n7\n",
    );
}

#[apply(policies)]
fn test_read_and_write(#[case] policy: IOPolicy) {
    policy.run(
        path_to_revbytes(),
        &b"\x1F\x8B\x08\x08\x0B\xC1\xA0\x62\x00\x03\x68\x69\x2E\x74\x78\x74\x00\xF3\xC8\xE4\x02\x00\x9A\x3C\x22\xD5\x03\x00\x00\x00"[..],
        &b"\x00\x00\x00\x03\xd5\x22\x3C\x9a\x00\x02\xe4\xc8\xf3\x00\x74\x78\x74\x2e\x69\x68\x03\x00\x62\xa0\xc1\x0b\x08\x08\x8b\x1f"[..],
    )
}
