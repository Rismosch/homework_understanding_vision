mod pcg;
mod rng;

use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use crate::rng::Rng;
use crate::rng::Seed;

const PATH_C: &str = "vp004rb_C.csv";
const PATH_O: &str = "vp004rb_O.csv";
const PATH_CO: &str = "vp004rb_CO.csv";
const PATHS: &[&str] = &[PATH_C, PATH_O, PATH_CO];

const PROBABILITY_WINDOW_SIZE: f64 = 0.05;

fn cmpf(lhs: &f64, rhs: &f64) -> std::cmp::Ordering {
    if *lhs < *rhs {
        std::cmp::Ordering::Less
    } else if *lhs > *rhs {
        std::cmp::Ordering::Greater
    } else {
        std::cmp::Ordering::Equal
    }
}

fn main() {
    // 6.
    let mut data = Vec::new();

    let mut prob = Vec::new();
    let mut acc = Vec::new();

    for path in PATHS.iter() {
        let bytes = std::fs::read(path).expect("no io error");
        let content = String::from_utf8(bytes).expect("valid utf8");
        let numbers = content
            .lines()
            .map(|x| x.parse::<f64>())
            .collect::<Result<Vec<_>, _>>()
            .expect("properly formatted floating point numbers in string");

        let mut sorted = numbers.clone();
        sorted.sort_by(cmpf);
        data.push(sorted.clone());

        let (p1, p2) = compute_prob_and_acc(&sorted);

        prob.push(p1);
        acc.push(p2);
    }

    let mut gnuplot = Command::new("gnuplot")
        .stdin(Stdio::piped())
        .spawn()
        .expect("gnuplot to spawn");
    let stdin = gnuplot.stdin.as_mut().expect("stdin to exist");

    writeln!(stdin, "set style line 1 lc rgb \"#FF0000\"").unwrap();
    writeln!(stdin, "set style line 2 lc rgb \"#00FF00\"").unwrap();
    writeln!(stdin, "set style line 3 lc rgb \"#0000FF\"").unwrap();
    writeln!(stdin, "set term png").unwrap();

    writeln!(
        stdin,
        "set output \"homework_3_6_probability_distribution_function.png\""
    )
    .unwrap();
    writeln!(stdin, "set title \"Probability distribution function\"").unwrap();
    writeln!(stdin, "set xlabel \"RT\"").unwrap();
    writeln!(stdin, "set ylabel \"Probability\"").unwrap();
    write!(stdin, "plot").unwrap();
    write!(stdin, " '-' with lines ls 1 title \"RT_C\", ").unwrap();
    write!(stdin, " '-' with lines ls 2 title \"RT_O\", ").unwrap();
    write!(stdin, " '-' with lines ls 3 title \"RT_{{CO}}\"").unwrap();
    writeln!(stdin).unwrap();
    for p in prob {
        for (x, y) in p {
            writeln!(stdin, "{} {}", x, y).unwrap();
        }
        writeln!(stdin, "e").unwrap();
    }

    writeln!(
        stdin,
        "set output \"homework_3_6_cumulative_distribution_function.png\""
    )
    .unwrap();
    writeln!(stdin, "set title \"Cumulative distribution function\"").unwrap();
    writeln!(stdin, "set xlabel \"RT\"").unwrap();
    writeln!(stdin, "set ylabel \"Cumulative probability\"").unwrap();
    write!(stdin, "plot").unwrap();
    write!(stdin, " '-' with lines ls 1 title \"RT_C\", ").unwrap();
    write!(stdin, " '-' with lines ls 2 title \"RT_O\", ").unwrap();
    write!(stdin, " '-' with lines ls 3 title \"RT_{{CO}}\"").unwrap();
    writeln!(stdin).unwrap();
    for p in acc {
        for (x, y) in p {
            writeln!(stdin, "{} {}", x, y).unwrap();
        }
        writeln!(stdin, "e").unwrap();
    }

    // 7. + 8.
    let data_rt_c = data[0].clone();
    let data_rt_o = data[1].clone();
    let data_rt_co = data[2].clone();

    let mut data_rt_co_race = Vec::new();
    let mut rng = Rng::new(Seed::default());

    for _ in 0..10000 {
        let rt_c = *rng.next_in(&data_rt_c);
        let rt_o = *rng.next_in(&data_rt_o);
        let rt_co = f64::min(rt_c, rt_o);
        data_rt_co_race.push(rt_co);
    }

    data_rt_co_race.sort_by(cmpf);

    let (prob1, acc1) = compute_prob_and_acc(&data_rt_co);
    let (prob2, acc2) = compute_prob_and_acc(&data_rt_co_race);

    let prob = vec![prob1, prob2];
    let acc = vec![acc1, acc2];

    writeln!(stdin, "set style line 1 lc rgb \"#0000FF\"").unwrap();
    writeln!(stdin, "set style line 2 lc rgb \"#FF00FF\"").unwrap();

    writeln!(
        stdin,
        "set output \"homework_3_8_probability_distribution_function.png\""
    )
    .unwrap();
    writeln!(stdin, "set title \"Probability distribution function\"").unwrap();
    writeln!(stdin, "set xlabel \"RT\"").unwrap();
    writeln!(stdin, "set ylabel \"Probability\"").unwrap();
    write!(stdin, "plot").unwrap();
    write!(stdin, " '-' with lines ls 1 title \"RT_{{CO}}\", ").unwrap();
    write!(
        stdin,
        " '-' with lines ls 2 title \"RT_{{CO}} (race model)\""
    )
    .unwrap();
    writeln!(stdin).unwrap();
    for p in &prob {
        for (x, y) in p {
            writeln!(stdin, "{} {}", x, y).unwrap();
        }
        writeln!(stdin, "e").unwrap();
    }

    writeln!(
        stdin,
        "set output \"homework_3_8_cumulative_distribution_function.png\""
    )
    .unwrap();
    writeln!(stdin, "set title \"Cumulative distribution function\"").unwrap();
    writeln!(stdin, "set xlabel \"RT\"").unwrap();
    writeln!(stdin, "set ylabel \"Cumulative probability\"").unwrap();
    write!(stdin, "plot").unwrap();
    write!(stdin, " '-' with lines ls 1 title \"RT_{{CO}}\", ").unwrap();
    write!(
        stdin,
        " '-' with lines ls 2 title \"RT_{{CO}} (race model)\""
    )
    .unwrap();
    writeln!(stdin).unwrap();
    for p in acc {
        for (x, y) in p {
            writeln!(stdin, "{} {}", x, y).unwrap();
        }
        writeln!(stdin, "e").unwrap();
    }

    // 9.
    let rt_co = &prob[0];
    let rt_race = &prob[1];

    // most likely
    println!();
    println!("max");

    let mut max_x = 0.0;
    let mut max_y = f64::MIN;

    for &(x, y) in rt_co.iter() {
        if y > max_y {
            max_x = x;
            max_y = y;
        }
    }

    println!("RC_{{CO}}       : {}", max_x);

    let mut max_x = 0.0;
    let mut max_y = f64::MIN;

    for &(x, y) in rt_race.iter() {
        if y > max_y {
            max_x = x;
            max_y = y;
        }
    }

    println!("RC_{{CO}}^{{race}}: {}", max_x);

    // average RT
    println!();
    println!("average");

    let mut sum = 0.0;
    for &x in data_rt_co.iter() {
        sum += x;
    }
    let average = sum / data_rt_co.len() as f64;
    println!("RC_{{CO}}       : {}", average);

    let mut sum = 0.0;
    for &x in data_rt_co_race.iter() {
        sum += x;
    }
    let average = sum / data_rt_co_race.len() as f64;
    println!("RC_{{CO}}^{{race}}: {}", average);


    // median RT
    println!();
    println!("median");

    let mut sorted = data_rt_co.clone();
    sorted.sort_by(cmpf);
    let median = sorted[sorted.len() / 2];
    println!("RC_{{CO}}       : {}", median);

    let mut sorted = data_rt_co_race.clone();
    sorted.sort_by(cmpf);
    let median = sorted[sorted.len() / 2];
    println!("RC_{{CO}}^{{race}}: {}", median);
}

fn compute_prob_and_acc(data: &[f64]) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
    let mut min = 0.0;
    loop {
        let candidate = min + PROBABILITY_WINDOW_SIZE;
        if candidate > data[0] {
            break;
        }

        min = candidate;
    }

    let mut prob = Vec::new();
    let mut acc = Vec::<(f64, f64)>::new();

    let mut current = min;
    let mut count = 1;
    let mut values = data.iter();
    loop {
        let Some(&next) = values.next() else {
            let x = current;
            let y1 = count as f64 / data.len() as f64;
            let y2 = y1 + acc.last().map(|last| last.1).unwrap_or(0.0);
            prob.push((x, y1));
            acc.push((x, y2));

            break;
        };

        if next < current + PROBABILITY_WINDOW_SIZE {
            count += 1;
            continue;
        } else {
            let x = current;
            let y1 = count as f64 / data.len() as f64;
            let y2 = y1 + acc.last().map(|last| last.1).unwrap_or(0.0);
            prob.push((x, y1));
            acc.push((x, y2));

            current = next;
            count = 1;
        }
    }

    (prob, acc)
}
