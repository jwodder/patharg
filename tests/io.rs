extern crate rstest_reuse;
use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use rstest::rstest;
use rstest_reuse::{apply, template};
use test_binary::build_test_binary_once;

build_test_binary_once!(linelen, "tests/bins");

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
    fn run(&self, input: &'static str, output: &'static str) {
        let mut cmd = Command::new(path_to_linelen());
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
                tmpfile.write_str(input).unwrap();
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
                r.stdout(output);
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
