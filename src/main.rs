use std::collections::HashMap;
use std::env;
use std::io;

use io::BufRead;
use io::Write;

fn matrix_to_edges<'a>(labels: &[&'a str], matrix: &[Vec<i32>]) -> Vec<(&'a str, &'a str, i32)> {
    let mut edges = Vec::with_capacity(matrix.len());
    for (i, row) in matrix.iter().enumerate() {
        for (j, &w) in row.iter().enumerate() {
            if w != 0 {
                edges.push((labels[i], labels[j], w))
            }
        }
    }
    edges
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let ascii = args.iter().any(|arg| arg == "--ascii");

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut label = String::new();
    handle.read_line(&mut label)?;
    let mut matrix = Vec::new();
    for line in handle.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split(',').skip(1).map(str::trim).collect();
        let weights = fields
            .iter()
            .map(|f| {
                f.parse::<i32>().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("{e}: '{f}' in the line '{fields:?}'"),
                    )
                })
            })
            .collect::<io::Result<Vec<i32>>>()?;
        matrix.push(weights);
    }

    let labels: Vec<&str> = label.split(',').skip(1).map(str::trim).collect();
    let edges = matrix_to_edges(&labels, &matrix);
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    if ascii {
        let n = labels.len();
        let size = ((n * 4) | 1).max(7);
        let cx = size / 2;
        let cy = size / 2;
        let r = (size / 2 - 1) as f64;

        let vertices: HashMap<&str, (usize, usize)> = (0..n)
            .map(|i| {
                let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
                let x = (cx as f64 + r * angle.cos()) as usize;
                let y = (cy as f64 + r * angle.sin()) as usize;
                (labels[i], (x, y))
            })
            .collect();

        let mut map: Vec<Vec<&str>> = vec![vec![" "; size]; size];

        for &(f, t, _w) in &edges {
            let (x0, y0) = vertices[f];
            let (x1, y1) = vertices[t];
            let dx = x1 as isize - x0 as isize;
            let dy = y1 as isize - y0 as isize;
            // This may produce fewer steps than needed, but for undirected graphs
            // the reverse edge (B→A) will draw the same line with correct steps.
            let steps = dx.max(dy);
            for s in 1..steps {
                let x = (x0 as isize + dx * s / steps) as usize;
                let y = (y0 as isize + dy * s / steps) as usize;
                map[y][x] = "*";
            }
        }

        for (&k, &(x, y)) in vertices.iter() {
            map[y][x] = k;
        }

        // ----   4 dash for a cell
        // | X |
        writeln!(handle, "{}", "-".repeat(4 * size))?;
        for row in &map {
            write!(handle, "|")?;
            for &c in row {
                write!(handle, " {} |", c)?;
            }
            writeln!(handle)?;
            writeln!(handle, "{}", "-".repeat(4 * size))?;
        }
    } else {
        handle.write_all(b"from,to,weight\n")?;
        for (f, t, w) in edges.iter() {
            handle.write_fmt(format_args!("{f},{t},{w}\n"))?;
        }
    }
    handle.flush()?;

    Ok(())
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    /// 3 nodes, sparse (A-B-C line), weighted
    /// ,A,B,C
    /// A,0,3,0
    /// B,3,0,4
    /// C,0,4,0
    #[test]
    fn test_sparse() {
        let labels = vec!["A", "B", "C"];
        let matrix = vec![
            vec![0, 3, 0],
            vec![3, 0, 4],
            vec![0, 4, 0],
        ];
        let result = matrix_to_edges(&labels,&matrix);
        let expected = vec![
            ("A", "B", 3),
            ("B", "A", 3),
            ("B", "C", 4),
            ("C", "B", 4),
        ];
        assert_eq!(result, expected);
    }

    /// 4 nodes, fully connected, weighted
    /// ,W,X,Y,Z
    /// W,0,3,5,7
    /// X,3,0,1,4
    /// Y,5,1,0,2
    /// Z,7,4,2,0
    #[test]
    fn test_fully_connected() {
        let labels = vec!["W", "X", "Y", "Z"];
        let matrix = vec![
            vec![0, 3, 5, 7],
            vec![3, 0, 1, 4],
            vec![5, 1, 0, 2],
            vec![7, 4, 2, 0],
        ];
        let result = matrix_to_edges(&labels,&matrix);
        let expected = vec![
            ("W", "X", 3),
            ("W", "Y", 5),
            ("W", "Z", 7),
            ("X", "W", 3),
            ("X", "Y", 1),
            ("X", "Z", 4),
            ("Y", "W", 5),
            ("Y", "X", 1),
            ("Y", "Z", 2),
            ("Z", "W", 7),
            ("Z", "X", 4),
            ("Z", "Y", 2),
        ];
        assert_eq!(result, expected);
    }

    /// 1 node, no edges
    /// ,A
    /// A,0
    #[test]
    fn test_single_node() {
        let labels = vec!["A"];
        let matrix = vec![vec![0]];
        let result = matrix_to_edges(&labels,&matrix);
        let expected: Vec<(&str, &str, i32)> = vec![];
        assert_eq!(result, expected);
    }

    /// 5 nodes, undirected, unweighted (ring: A-B-C-D-E-A)
    /// ,A,B,C,D,E
    /// A,0,1,0,0,1
    /// B,1,0,1,0,0
    /// C,0,1,0,1,0
    /// D,0,0,1,0,1
    /// E,1,0,0,1,0
    #[test]
    fn test_ring_graph() {
        let labels = vec!["A", "B", "C", "D", "E"];
        let matrix = vec![
            vec![0, 1, 0, 0, 1],
            vec![1, 0, 1, 0, 0],
            vec![0, 1, 0, 1, 0],
            vec![0, 0, 1, 0, 1],
            vec![1, 0, 0, 1, 0],
        ];
        let result = matrix_to_edges(&labels,&matrix);
        let expected = vec![
            ("A", "B", 1),
            ("A", "E", 1),
            ("B", "A", 1),
            ("B", "C", 1),
            ("C", "B", 1),
            ("C", "D", 1),
            ("D", "C", 1),
            ("D", "E", 1),
            ("E", "A", 1),
            ("E", "D", 1),
        ];
        assert_eq!(result, expected);
    }
}
