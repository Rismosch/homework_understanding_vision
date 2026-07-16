use std::io::{stdin, Write};
use std::process::{Command, Stdio};

const DATA_PATH: &str = "ConeSensitivity_Function_ForExercise2024.csv";

#[derive(Debug, Clone, Copy)]
struct Row {
    wavelength: usize,
    s_cone_sensitivity: f64,
    m_cone_sensitivity: f64,
    l_cone_sensitivity: f64,
}

#[derive(Debug, Clone, Copy)]
enum Cone {
    S,
    M,
    L,
}

#[derive(Debug, Clone, Copy)]
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
    writeln!(stdin, "set title \"Cone sensitivity spectrum\"").unwrap();
    writeln!(stdin, "set xlabel \"Wavelength {{/Symbol l}} (nm)\"").unwrap();
    writeln!(stdin, "set ylabel \"Cone sensitivity f_a({{/Symbol l}})\"").unwrap();
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
    let cones = [Cone::S, Cone::M, Cone::L];

    //let x_values = (1usize..256usize).collect::<Vec<_>>();
    let x_values = (0usize..=15usize).collect::<Vec<_>>();
    let mut cone_values = Vec::with_capacity(cones.len());

    let mut lambdas = Vec::with_capacity(cones.len());
    for &cone in cones.iter() {
        let mut y_values = Vec::with_capacity(x_values.len());

        let s = (570, 1).into();
        let lambda = mean_r_a(&rows, cone, s)
            .expect(&format!("mean r_a for wavelength {} to exist", s.wavelength));
        lambdas.push(lambda);

        for &x in x_values.iter() {
            let y = poisson_distribution(lambda, x);
            y_values.push(y);
        }

        cone_values.push(y_values);
    }

    writeln!(stdin,"set output \"homework_4_B_cone_absorption_likelihood.png\"").unwrap();
    writeln!(stdin, "set title \"Cone absorption likelihood\"").unwrap();
    writeln!(stdin, "set xlabel \"Cone absorption r_a\"").unwrap();
    writeln!(stdin, "set ylabel \"Likelihood P(r_a | S = ({{/Symbol l}}, I))\"").unwrap();
    writeln!(stdin, "unset logscale y").unwrap();
    writeln!(stdin, "set format y").unwrap();

    for (i, &lambda) in lambdas.iter().enumerate() {
        let n = i + 1;
        let mean = lambda;
        writeln!(stdin, "set arrow {} from {}, graph 0 to {}, graph 1 nohead ls {}", n, mean, mean, n).unwrap();

        writeln!(stdin, "set label {} \"mean\" at {}, graph 1 offset 0,-1 tc ls {}", n, mean, n).unwrap();
    }

    write!(stdin, "plot").unwrap();
    write!(stdin, " '-' w lp ls 1 pt 2 title \"S\", ").unwrap();
    write!(stdin, " '-' w lp ls 2 pt 2 title \"M\", ").unwrap();
    write!(stdin, " '-' w lp ls 3 pt 2 title \"L\"").unwrap();
    writeln!(stdin).unwrap();

    for y_values in cone_values.iter() {
        for (i, &y) in y_values.iter().enumerate() {
            let x = x_values[i];
            writeln!(stdin, "{} {}", x, y).unwrap();
        }

        writeln!(stdin, "e").unwrap();
    }
}

fn mean_r_a(rows: &[Row], a: Cone, s: S) -> Option<f64> {
    for row in rows {
        if row.wavelength == s.wavelength {
            return Some(s.I as f64 * row.sensitivity(a));
        }
    }

    return None;
}

fn poisson_distribution(lambda: f64, k: usize) -> f64 {
    let mut fract = lambda.powf(k as f64) * f64::exp(-lambda);

    for x in 2..=k {
        fract /= x as f64;
    }

    fract
}
