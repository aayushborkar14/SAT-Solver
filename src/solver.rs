use crate::wff::Clause;
use crate::wff::Formula;
use crate::wff::Literal;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Clone)]
pub struct Assignment {
    pub value: bool,
    pub antecedent: Option<Clause>,
    pub dl: i32,
}

impl Assignment {
    pub fn new(value: bool, antecedent: Option<Clause>, dl: i32) -> Assignment {
        Assignment {
            value,
            antecedent,
            dl,
        }
    }
}

pub struct Assignments {
    pub assignments: HashMap<String, Assignment>,
    dl: i32,
}

impl Assignments {
    pub fn new() -> Assignments {
        Assignments {
            assignments: HashMap::new(),
            dl: 0,
        }
    }

    pub fn assign(&mut self, variable: String, value: bool, antecedent: Option<Clause>) {
        let assignment = Assignment::new(value, antecedent, self.dl);
        self.assignments.insert(variable, assignment);
    }

    pub fn remove(&mut self, variable: &String) {
        self.assignments.remove(variable);
    }

    pub fn get(&self, variable: &String) -> Option<&Assignment> {
        self.assignments.get(variable)
    }
}

pub enum SolverResult {
    Satisfied,
    Unresolved,
}

pub struct CdclSolver {
    pub formula: Formula,
    assignments: Assignments,
    sat: SolverResult,
}

pub enum ClauseStatus {
    Satisfied,
    Unsatisfied,
    Unit,
    Unresolved,
}

pub enum UnitPropagationResult {
    Conflict,
    Unresolved,
}

impl CdclSolver {
    pub fn new(formula: Formula) -> CdclSolver {
        CdclSolver {
            formula,
            assignments: Assignments::new(),
            sat: SolverResult::Unresolved,
        }
    }

    pub fn assignments(&self) -> &Assignments {
        &self.assignments
    }

    pub fn sat(&self) -> &SolverResult {
        &self.sat
    }

    pub fn solve(&mut self) {
        let (reason, _) = self.unit_propagation();
        if matches!(reason, UnitPropagationResult::Conflict) {
            return;
        }

        while !self.all_variables_assigned() {
            let (var, val) = self.pick_branching_variable();
            println!("Guessing {} = {}", var, val);
            println!("Decision level: {}", self.assignments.dl);
            self.assignments.dl += 1;
            self.assignments.assign(var, val, None);

            loop {
                let (reason, clause) = self.unit_propagation();
                if !matches!(reason, UnitPropagationResult::Conflict) {
                    break;
                }

                let (b, learnt_clause) = self.conflict_analysis(clause.as_ref().unwrap());
                if b < 0 {
                    return;
                }

                if let Some(learnt) = learnt_clause {
                    self.add_learned_clause(learnt);
                }
                self.backtrack(b);
                self.assignments.dl = b;
                println!("Backtracked to decision level {}", b);
            }
        }
        self.sat = SolverResult::Satisfied;
    }

    pub fn clause_status(&self, clause: &Clause) -> ClauseStatus {
        let mut false_count: i32 = 0;
        let mut true_count: i32 = 0;
        for literal in &clause.literals {
            match self.assignments.get(&literal.value) {
                Some(assignment) => {
                    if assignment.value == literal.negation {
                        false_count += 1;
                    } else {
                        true_count += 1;
                    }
                }
                None => {}
            }
        }
        if true_count > 0 {
            return ClauseStatus::Satisfied;
        } else if false_count == clause.literals.len() as i32 {
            return ClauseStatus::Unsatisfied;
        } else if false_count == clause.literals.len() as i32 - 1 {
            return ClauseStatus::Unit;
        } else {
            return ClauseStatus::Unresolved;
        }
    }

    pub fn unit_propagation(&mut self) -> (UnitPropagationResult, Option<Clause>) {
        let mut finished: bool = false;
        while !finished {
            finished = true;
            for clause in &self.formula.clauses {
                match self.clause_status(clause) {
                    ClauseStatus::Satisfied | ClauseStatus::Unresolved => {}
                    ClauseStatus::Unsatisfied => {
                        return (UnitPropagationResult::Conflict, Some(clause.clone()));
                    }
                    ClauseStatus::Unit => {
                        finished = false;
                        let mut unit_literal: Option<Literal> = None;
                        for literal in &clause.literals {
                            match self.assignments.get(&literal.value) {
                                Some(_) => {}
                                None => {
                                    unit_literal = Some(literal.clone());
                                    break;
                                }
                            }
                        }
                        match unit_literal {
                            Some(literal) => {
                                println!(
                                    "Unit propagation, assigning {} = {}",
                                    literal.value, !literal.negation
                                );

                                self.assignments.assign(
                                    literal.value.clone(),
                                    !literal.negation,
                                    Some(clause.clone()),
                                );
                                println!("Decision level: {}", self.assignments.dl);
                            }
                            None => {}
                        }
                    }
                }
            }
        }
        return (UnitPropagationResult::Unresolved, None);
    }

    pub fn add_learned_clause(&mut self, clause: Clause) {
        self.formula.clauses.push(clause);
    }

    pub fn all_variables_assigned(&self) -> bool {
        return self.assignments.assignments.len() == self.formula.variables.len();
    }

    pub fn pick_branching_variable(&self) -> (String, bool) {
        let assigned_vars: HashSet<String> = self.assignments.assignments.keys().cloned().collect();

        let unassigned_variables: Vec<&String> =
            self.formula.variables.difference(&assigned_vars).collect();

        let mut rng = rand::thread_rng();

        let random_bool: bool = rng.gen();
        let random_variable: &String = unassigned_variables.choose(&mut rng).unwrap();

        (random_variable.clone(), random_bool)
    }

    pub fn backtrack(&mut self, b: i32) {
        let mut to_remove: Vec<String> = Vec::new();
        for (variable, assignment) in &self.assignments.assignments {
            if assignment.dl > b {
                to_remove.push(variable.clone());
            }
        }
        for variable in to_remove {
            println!("Backtracking, removing assignment for {}", variable);
            self.assignments.remove(&variable);
        }
    }

    pub fn resolve(&self, a: &Clause, b: &Clause, x: &str) -> Clause {
        let mut result: HashSet<Literal> = a.literals.iter().cloned().collect();
        result.extend(b.literals.iter().cloned());
        result.remove(&Literal::new(x.to_string(), true));
        result.remove(&Literal::new(x.to_string(), false));
        Clause::new(result.into_iter().collect())
    }

    pub fn conflict_analysis(&self, clause: &Clause) -> (i32, Option<Clause>) {
        if self.assignments.dl == 0 {
            return (-1, None);
        }

        let mut current_clause = clause.clone();
        let mut literals: Vec<Literal> = current_clause
            .literals
            .iter()
            .filter(|lit| self.assignments.get(&lit.value).unwrap().dl == self.assignments.dl)
            .cloned()
            .collect();

        while literals.len() != 1 {
            literals = literals
                .into_iter()
                .filter(|lit| {
                    self.assignments
                        .get(&lit.value)
                        .unwrap()
                        .antecedent
                        .is_some()
                })
                .collect();

            if let Some(literal) = literals.first() {
                let antecedent = self
                    .assignments
                    .get(&literal.value)
                    .unwrap()
                    .antecedent
                    .as_ref()
                    .unwrap();
                current_clause = self.resolve(&current_clause, antecedent, &literal.value);

                literals = current_clause
                    .literals
                    .iter()
                    .filter(|lit| {
                        self.assignments.get(&lit.value).unwrap().dl == self.assignments.dl
                    })
                    .cloned()
                    .collect();
            } else {
                // Handle the case where no literal meets the criteria
                break;
            }
        }

        let mut decision_levels: Vec<i32> = current_clause
            .literals
            .iter()
            .map(|lit| self.assignments.get(&lit.value).unwrap().dl)
            .collect::<HashSet<i32>>()
            .into_iter()
            .collect();

        decision_levels.sort_unstable();

        if decision_levels.len() <= 1 {
            (0, Some(current_clause))
        } else {
            (
                *decision_levels.iter().rev().nth(1).unwrap(),
                Some(current_clause),
            )
        }
    }
}
