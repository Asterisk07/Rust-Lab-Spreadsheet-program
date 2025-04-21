use rand::seq::SliceRandom;
use rand::thread_rng;
use std::env;
use std::fs::File;
use std::io::Write;

mod convert {
    pub fn num_to_alpha(num: u32) -> String {
        let mut n = num;
        let mut result = Vec::new();
        while n > 0 {
            n -= 1;
            let c = (b'A' + (n % 26) as u8) as char;
            result.push(c);
            n /= 26;
        }
        result.iter().rev().collect()
    }
}

fn get_tree(n: usize, m: usize) -> Vec<String> {
    let k = n * m;
    let mut perm: Vec<usize> = (0..k).collect();
    let mut rng = thread_rng();
    perm.shuffle(&mut rng);

    let mut parent = vec![0; k];
    for i in 1..k {
        parent[perm[i]] = perm[rand::Rng::gen_range(&mut rng, 0..i)];
    }

    let mut result = Vec::with_capacity(k);
    for i in 1..k {
        let cell = perm[i];
        let parent_cell = parent[perm[i]];
        
        let col = (cell % m) + 1;
        let row = (cell / m) + 1;
        let pcol = (parent_cell % m) + 1;
        let prow = (parent_cell / m) + 1;

        let mut s = convert::num_to_alpha(col as u32);
        s.push_str(&row.to_string());
        s.push('=');
        s.push_str(&convert::num_to_alpha(pcol as u32));
        s.push_str(&prow.to_string());
        s.push_str("+1");
        
        result.push(s);
    }

    let root = perm[0];
    let root_col = (root % m) + 1;
    let root_row = (root / m) + 1;
    result.push(format!("{}{}=100", convert::num_to_alpha(root_col as u32), root_row));

    result
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} N M filename", args[0]);
        std::process::exit(1);
    }

    let n = args[1].parse::<usize>().unwrap_or_else(|_| {
        eprintln!("Invalid N");
        std::process::exit(1);
    });

    let m = args[2].parse::<usize>().unwrap_or_else(|_| {
        eprintln!("Invalid M");
        std::process::exit(1);
    });

    let tree = get_tree(n, m);
    let mut file = File::create(&args[3])?;
    
    for line in tree {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}