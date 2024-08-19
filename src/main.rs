use solver::CdclSolver;
use solver::SolverResult;
use std::env;
use std::fs;
use std::process;
use wff::Clause;
use wff::Formula;
use wff::Literal;

mod solver;
mod tseitin;
mod wff;

pub fn parse_dimacs_cnf(content: &str) -> Formula {
    let mut clauses = vec![Clause::new(Vec::new())];

    for line in content.lines() {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if !tokens.is_empty() && tokens[0] != "p" && tokens[0] != "c" {
            for tok in tokens {
                if let Ok(lit) = tok.parse::<i32>() {
                    if lit == 0 {
                        clauses.push(Clause::new(Vec::new()));
                    } else {
                        let var = lit.abs().to_string();
                        let neg = lit < 0;
                        clauses
                            .last_mut()
                            .unwrap()
                            .literals
                            .push(Literal::new(var, neg));
                    }
                }
            }
        }
    }

    if clauses.last().unwrap().literals.is_empty() {
        clauses.pop();
    }

    Formula::new(clauses)
}

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Provide one DIMACS CNF filename as argument.");
        process::exit(1);
    }

    let filename = &args[1];
    let dimacs_cnf = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            process::exit(1);
        }
    };

    let formula = parse_dimacs_cnf(&dimacs_cnf);
    let mut solver = CdclSolver::new(formula);
    solver.solve();
    let result = solver.sat();

    match result {
        SolverResult::Satisfied => {
            println!("Formula is SAT with assignments:");
            for (var, assignment) in solver.assignments().assignments.iter() {
                println!("{}: {}", var, assignment.value);
            }
        }
        SolverResult::Unresolved => {
            println!("Formula is UNSAT.");
        }
    }
}
