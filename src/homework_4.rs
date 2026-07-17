use std::io::Write;
use std::ops::Range;
use std::process::{ChildStdin, Command, Stdio};
use std::usize;

use crate::rng::{Rng, Seed};

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

impl Cone {
    fn to_index(self) -> usize {
        match self {
            Cone::S => 0,
            Cone::M => 1,
            Cone::L => 2,
        }
    }
}

#[derive(Debug, Clone)]
struct S {
    wavelength: usize,
    I: usize,
    x_window: Range<usize>,
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

const RED: &str =       "#FF0000";
const GREEN: &str =     "#00FF00";
const BLUE: &str =      "#0000FF";
const CYAN: &str =      "#00FF00";
const MAGENTA: &str =   "#FF00FF";
const YELLOW: &str =    "#FFFF00";

fn reset_gnuplot(stdin: &mut ChildStdin, colors: impl AsRef<[&'static str]>) {
    writeln!(stdin, "reset").unwrap();

    for (i, &color) in colors.as_ref().iter().enumerate() {
        writeln!(stdin, "set style line {} lc rgb \"{}\"", i + 1, color).unwrap();
    }

    writeln!(stdin, "set terminal pngcairo enhanced").unwrap();
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

    // init rng
    let mut rng = Rng::new(Seed::new());

    // open gnuplot
    let mut gnuplot = Command::new("gnuplot")
        .stdin(Stdio::piped())
        .spawn()
        .expect("gnuplot to spawn");
    let stdin = gnuplot.stdin.as_mut().expect("stdin to exist");


    // A
    reset_gnuplot(stdin, [RED, GREEN, BLUE]);
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


    // I
    let scenes: &[S] = &[
        S{
            wavelength: 570,
            I: 100,
            x_window: 0..800,
        },
        S{
            wavelength: 470,
            I: 100,
            x_window: 0..100,
        },
    ];

    for s in scenes.iter() {
        let S { wavelength, I, x_window } = s.clone();

        // B
        let cones = [Cone::S, Cone::M, Cone::L];

        let x_values = x_window.clone().collect::<Vec<_>>();
        let mut cone_values = Vec::with_capacity(cones.len());

        let mut lambdas = Vec::with_capacity(cones.len());
        for &cone in cones.iter() {
            let mut y_values = Vec::with_capacity(x_values.len());

            let lambda = mean_r_a(&rows, cone, s)
                .expect(&format!("mean r_a for wavelength {} to exist", s.wavelength));
            lambdas.push(lambda);

            for &x in x_values.iter() {
                let y = poisson_distribution(lambda, x);
                y_values.push(y);
            }

            cone_values.push(y_values);
        }

        reset_gnuplot(stdin, [RED, GREEN, BLUE]);
        writeln!(stdin,"set output \"homework_4_B_cone_absorption_likelihood_w={}_I={}.png\"", wavelength, I).unwrap();
        writeln!(stdin, "set title \"Cone absorption likelihood ({{/Symbol l}} = {})\"", wavelength).unwrap();
        writeln!(stdin, "set xlabel \"Cone absorption r_a\"").unwrap();
        writeln!(stdin, "set ylabel \"Likelihood P(r_a | S = ({{/Symbol l}}, I))\"").unwrap();
        //writeln!(stdin, "set logscale y").unwrap();
        writeln!(stdin, "set format y").unwrap();

        for (i, &lambda) in lambdas.iter().enumerate() {
            let n = i + 1;
            let mean = lambda;
            writeln!(stdin, "set arrow {} from {}, graph 0 to {}, graph 1 nohead ls {}", n, mean, mean, n).unwrap();

            writeln!(stdin, "set label {} \"mean\" at {}, graph 1 offset 0,-1 tc ls {}", n, mean, n).unwrap();
        }

        write!(stdin, "plot").unwrap();
        //write!(stdin, " '-' w lp ls 1 pt 2 title \"S\", ").unwrap();
        //write!(stdin, " '-' w lp ls 2 pt 2 title \"M\", ").unwrap();
        //write!(stdin, " '-' w lp ls 3 pt 2 title \"L\"").unwrap();
        write!(stdin, " '-' w l ls 1 title \"S\", ").unwrap();
        write!(stdin, " '-' w l ls 2 title \"M\", ").unwrap();
        write!(stdin, " '-' w l ls 3 title \"L\"").unwrap();
        writeln!(stdin).unwrap();

        for y_values in cone_values.iter() {
            for (i, &y) in y_values.iter().enumerate() {
                let x = x_values[i];
                writeln!(stdin, "{} {}", x, y).unwrap();
            }

            writeln!(stdin, "e").unwrap();
        }

        // C
        let mut cummulations = Vec::with_capacity(cone_values.len());
        for i in 0..cummulations.capacity() {
            let y_values = &cone_values[i];
            let mut cummulation = Vec::with_capacity(y_values.len());
            for &y in y_values.iter() {
                let y = if y.is_infinite() {
                    0.0
                } else {
                    y
                };

                let prev = cummulation.last().copied().unwrap_or(0.0);
                cummulation.push(prev + y);
            }
            cummulations.push(cummulation);
        }

        let mut sample_cummulation = |c: &[f64]| {
            let target = rng.next_f32() as f64;
            let mut index = None;
            for (i, &y) in c.iter().enumerate() {
                if y < target {
                    index = Some(i);
                } else {
                    break;
                }
            }

            index
        };

        let sample_count = 1000;
        let mut samples = Vec::with_capacity(sample_count);
        for _ in 0..samples.capacity() {
            let s = sample_cummulation(&cummulations[0]);
            let m = sample_cummulation(&cummulations[1]);
            let l = sample_cummulation(&cummulations[2]);
            let (Some(s), Some(m), Some(l)) = (s, m, l) else {
                continue;
            };

            samples.push((s, m, l));
        }

        reset_gnuplot(stdin, &[BLUE, MAGENTA]);
        writeln!(stdin,"set output \"homework_4_C_cone_responses_w={}_I={}.png\"", wavelength, I).unwrap();
        writeln!(stdin, "set title \"Cone Responses\"").unwrap();
        writeln!(stdin, "set xlabel \"r_S\"").unwrap();
        writeln!(stdin, "set ylabel \"r_M\"").unwrap();
        writeln!(stdin, "set zlabel \"r_L\"").unwrap();
        //writeln!(stdin, "set xrange [0:1]").unwrap();

        write!(stdin, "splot '-' u 1:2:3 w p ls 1 t \"{{/Symbol l}} = {}\"", wavelength).unwrap();
        writeln!(stdin).unwrap();
        for &(s, m, l) in samples.iter() {
            writeln!(stdin, "{} {} {}", s, m, l).unwrap();
        }
        writeln!(stdin, "e").unwrap();
    }
}

fn mean_r_a(rows: &[Row], a: Cone, s: &S) -> Option<f64> {
    for row in rows {
        if row.wavelength == s.wavelength {
            return Some(s.I as f64 * row.sensitivity(a));
        }
    }

    return None;
}

//fn poisson_distribution(lambda: f64, k: usize) -> f64 {
//    let nominator = lambda.powf(k as f64) * f64::exp(-lambda);
//
//    let mut result = nominator;
//    for x in 2..=k {
//        result /= x as f64;
//    }
//
//    result as f64
//}

fn poisson_distribution(lambda: f64, k: usize) -> f64 {
    let mut factorial = f64::ln(k as f64);
    for i in 2..k {
        factorial += f64::ln(i as f64);
    }

    let ln_distribution = (k as f64) * f64::ln(lambda) - factorial - lambda;
    f64::exp(ln_distribution)
}

//fn poisson_distribution(lambda: f64, k: usize) -> f64 {
//    //let nominator = lambda.powf(k as f64) * f64::exp(-lambda);
//    let nominator = (lambda as f128).powf(k as f128) / f128::exp(lambda as f128);
//
//    let mut result = nominator;
//    for x in 2..=k {
//        result /= x as f128;
//    }
//
//    result as f64
//}
