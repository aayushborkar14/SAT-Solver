use std::collections::HashSet;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Literal {
    pub value: String,
    pub negation: bool,
}

impl Literal {
    pub fn new(value: String, negation: bool) -> Literal {
        Literal { value, negation }
    }

    pub fn negate(&self) -> Literal {
        return Literal {
            value: self.value.clone(),
            negation: !self.negation,
        };
    }

    pub fn to_string(&self) -> String {
        if self.negation {
            return format!("¬{}", self.value);
        } else {
            return self.value.clone();
        }
    }
}

#[derive(Clone)]
pub struct Clause {
    pub literals: Vec<Literal>,
}

impl Clause {
    pub fn new(literals: Vec<Literal>) -> Clause {
        Clause { literals }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (i, literal) in self.literals.iter().enumerate() {
            result.push_str(&literal.to_string());
            if i < self.literals.len() - 1 {
                result.push_str(" ∨ ");
            }
        }
        return result;
    }
}

#[derive(Clone)]
pub struct Formula {
    pub clauses: Vec<Clause>,
    pub variables: HashSet<String>,
}

impl Formula {
    pub fn new(clauses: Vec<Clause>) -> Formula {
        let mut variables = HashSet::new();
        for clause in &clauses {
            for literal in &clause.literals {
                variables.insert(literal.value.clone());
            }
        }
        Formula { clauses, variables }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (i, clause) in self.clauses.iter().enumerate() {
            result.push_str(&clause.to_string());
            if i < self.clauses.len() - 1 {
                result.push_str(" ∧ ");
            }
        }
        return result;
    }
}
