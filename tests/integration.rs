use std::io::Write;
use std::process::{Command, Stdio};

fn run_lsedge(input: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lsedge"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn lsedge");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .expect("failed to write to stdin");

    let output = child.wait_with_output().expect("failed to wait on child");
    assert!(
        output.status.success(),
        "lsedge failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("invalid utf8")
}

#[test]
fn test_sparse() {
    let input = "\
,A,B,C
A,0,3,0
B,3,0,4
C,0,4,0
";
    let expected = "\
from,to,weight
A,B,3
B,A,3
B,C,4
C,B,4
";
    assert_eq!(run_lsedge(input), expected);
}

#[test]
fn test_fully_connected() {
    let input = "\
,W,X,Y,Z
W,0,3,5,7
X,3,0,1,4
Y,5,1,0,2
Z,7,4,2,0
";
    let expected = "\
from,to,weight
W,X,3
W,Y,5
W,Z,7
X,W,3
X,Y,1
X,Z,4
Y,W,5
Y,X,1
Y,Z,2
Z,W,7
Z,X,4
Z,Y,2
";
    assert_eq!(run_lsedge(input), expected);
}

#[test]
fn test_single_node() {
    let input = "\
,A
A,0
";
    let expected = "\
from,to,weight
";
    assert_eq!(run_lsedge(input), expected);
}

#[test]
fn test_ring_graph() {
    let input = "\
,A,B,C,D,E
A,0,1,0,0,1
B,1,0,1,0,0
C,0,1,0,1,0
D,0,0,1,0,1
E,1,0,0,1,0
";
    let expected = "\
from,to,weight
A,B,1
A,E,1
B,A,1
B,C,1
C,B,1
C,D,1
D,C,1
D,E,1
E,A,1
E,D,1
";
    assert_eq!(run_lsedge(input), expected);
}
