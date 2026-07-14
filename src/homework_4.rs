use std::io::Write;
use std::process::{Command, Stdio};
use std::task::Wake;
use std::u128;

const DATA_PATH: &str = "ConeSensitivity_Function_ForExercise2024.csv";

#[derive(Debug, Clone, Copy)]
struct Row {
    wavelength: usize,
    s_cone_sensitivity: f64,
    m_cone_sensitivity: f64,
    l_cone_sensitivity: f64,
}

enum Cone {
    S,
    M,
    L,
}

struct S {
    wavelength: usize,
    I: usize,
}

impl Row {
    fn sensitivity(self, a: Cone) -> f64 {
        match a {
            Cone::S => self.s_cone_sensitivity,
            Cone::M => self.m_cone_sensitivity,
            Cone::L => self.l_cone_sensitivity,
        }
    }
}

impl From<(usize, usize)> for S {
    fn from(value: (usize, usize)) -> Self {
        Self {
            wavelength: value.0,
            I: value.1,
        }
    }
}

pub fn run() {
    // read file
    let file_content = std::fs::read_to_string(DATA_PATH).unwrap();

    let mut lines = file_content.lines();
    let _header = lines.next().expect("to have at least 1 line in the input");

    let rows = lines
        .map(|x|{
            let mut splits = x.split(';');
            let wavelength = splits
                .next()
                .expect("column 0 to exist")
                .parse()
                .expect("wavelength to have correct format");
            let s_cone_sensitivity = splits
                .next()
                .expect("column 1 to exist")
                .parse()
                .expect("s_cone_sensitivity to have correct format");
            let m_cone_sensitivity = splits
                .next()
                .expect("column 2 to exist")
                .parse()
                .expect("m_cone_sensitivity to have correct format");
            let l_cone_sensitivity = splits
                .next()
                .expect("column 3 to exist")
                .parse()
                .expect("l_cone_sensitivity to have correct format");

            Row {
                wavelength,
                s_cone_sensitivity,
                m_cone_sensitivity,
                l_cone_sensitivity,
            }
        })
        .collect::<Vec<_>>();

    // open gnuplot
    let mut gnuplot = Command::new("gnuplot")
        .stdin(Stdio::piped())
        .spawn()
        .expect("gnuplot to spawn");
    let stdin = gnuplot.stdin.as_mut().expect("stdin to exist");

    writeln!(stdin, "set style line 1 lc rgb \"#0000FF\"").unwrap();
    writeln!(stdin, "set style line 2 lc rgb \"#00FF00\"").unwrap();
    writeln!(stdin, "set style line 3 lc rgb \"#FF0000\"").unwrap();
    writeln!(stdin, "set terminal pngcairo enhanced").unwrap();

    // A
    writeln!(stdin,"set output \"homework_4_A_cone_sensitivity_spectrum.png\"").unwrap();
    writeln!(stdin, "set title \"Cone sensitivity spectrum \u{1D453}_\u{1D44E}({{/Symbol l}})\"").unwrap();
    writeln!(stdin, "set xlabel \"Wavelength {{/Symbol l}} (nm)\"").unwrap();
    writeln!(stdin, "set ylabel \"Cone sensitivity\"").unwrap();
    writeln!(stdin, "set logscale y").unwrap();
    writeln!(stdin, "set format y \"10^{{%L}}\"").unwrap();
    write!(stdin, "plot").unwrap();
    write!(stdin, " '-' with lines ls 1 title \"S\", ").unwrap();
    write!(stdin, " '-' with lines ls 2 title \"M\", ").unwrap();
    write!(stdin, " '-' with lines ls 3 title \"L\"").unwrap();
    writeln!(stdin).unwrap();

    for row in rows.iter() {
        writeln!(stdin, "{} {}", row.wavelength, row.s_cone_sensitivity).unwrap();
    }
    writeln!(stdin, "e").unwrap();
    for row in rows.iter() {
        writeln!(stdin, "{} {}", row.wavelength, row.m_cone_sensitivity).unwrap();
    }
    writeln!(stdin, "e").unwrap();
    for row in rows.iter() {
        writeln!(stdin, "{} {}", row.wavelength, row.l_cone_sensitivity).unwrap();
    }
    writeln!(stdin, "e").unwrap();


    // B
    poisson_distribution(
        &rows, 
        Cone::S,
        (100, 570).into(),
        20,
    );
}

fn mean_r_a(rows: &[Row], a: Cone, s: S) -> Option<f64> {
    for row in rows {
        if row.wavelength == s.wavelength {
            return Some(s.I as f64 * row.sensitivity(a));
        }
    }

    return None;
}

fn poisson_distribution(rows: &[Row], a: Cone, s: S, r_a: usize) {
    let mean = mean_r_a(rows, a, s).unwrap();

    let mut factorial = r_a as u128;
    for x in 1..r_a as u128 {
        factorial *= x;
    }

    let numerator = mean.powf(r_a as f64);
    let denominator = 1;

    todo!();
}
