use crate::wff::{Clause, Formula, Literal};
use std::collections::HashMap;

pub struct TseitinEncoder {
    variable_counter: usize,
    variable_map: HashMap<String, String>,
}

impl TseitinEncoder {
    pub fn new() -> Self {
        TseitinEncoder {
            variable_counter: 0,
            variable_map: HashMap::new(),
        }
    }

    fn new_variable(&mut self) -> String {
        self.variable_counter += 1;
        format!("t_{}", self.variable_counter)
    }

    pub fn encode(&mut self, formula: &Formula) -> Formula {
        let mut clauses = Vec::new();
        let root_var = self.encode_recursive(&formula.to_string(), &mut clauses);

        // Add the root variable as a unit clause
        clauses.push(Clause::new(vec![Literal::new(root_var, false)]));

        Formula::new(clauses)
    }

    fn encode_recursive(&mut self, subformula: &str, clauses: &mut Vec<Clause>) -> String {
        if let Some(var) = self.variable_map.get(subformula) {
            return var.clone();
        }

        let var = self.new_variable();
        self.variable_map
            .insert(subformula.to_string(), var.clone());

        if !subformula.contains('∧') && !subformula.contains('∨') {
            // Base case: literal
            let negated = subformula.starts_with('¬');
            let literal = if negated {
                Literal::new(subformula[1..].to_string(), false)
            } else {
                Literal::new(subformula.to_string(), false)
            };
            clauses.push(Clause::new(vec![
                Literal::new(var.clone(), true),
                literal.clone(),
            ]));
            clauses.push(Clause::new(vec![
                Literal::new(var.clone(), false),
                literal.negate(),
            ]));
        } else if subformula.contains('∧') {
            // AND operation
            let parts: Vec<&str> = subformula.split('∧').map(|s| s.trim()).collect();
            let left_var = self.encode_recursive(parts[0], clauses);
            let right_var = self.encode_recursive(parts[1], clauses);

            clauses.push(Clause::new(vec![
                Literal::new(var.clone(), true),
                Literal::new(left_var.clone(), false),
            ]));
            clauses.push(Clause::new(vec![
                Literal::new(var.clone(), true),
                Literal::new(right_var.clone(), false),
            ]));
            clauses.push(Clause::new(vec![
                Literal::new(var.clone(), false),
                Literal::new(left_var, true),
                Literal::new(right_var, true),
            ]));
        } else if subformula.contains('∨') {
            // OR operation
            let parts: Vec<&str> = subformula.split('∨').map(|s| s.trim()).collect();
            let left_var = self.encode_recursive(parts[0], clauses);
            let right_var = self.encode_recursive(parts[1], clauses);

            clauses.push(Clause::new(vec![
                Literal::new(var.clone(), false),
                Literal::new(left_var.clone(), true),
            ]));
            clauses.push(Clause::new(vec![
                Literal::new(var.clone(), false),
                Literal::new(right_var.clone(), true),
            ]));
            clauses.push(Clause::new(vec![
                Literal::new(var.clone(), true),
                Literal::new(left_var, false),
                Literal::new(right_var, false),
            ]));
        }

        var
    }
}
