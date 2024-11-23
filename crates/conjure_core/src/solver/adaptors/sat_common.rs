//! Common code for SAT adaptors.
//! Primarily, this is CNF related code.

use std::collections::HashMap;

use thiserror::Error;

use crate::{
    ast as conjure_ast, solver::SolverError, solver::SolverError::*, Model as ConjureModel,
};
// (nd60, march 24) - i basically copied all this from @gskorokod's SAT implemention for the old
// solver interface.
use crate::metadata::Metadata;

use super::sat_adaptor::CNFError;

/// A representation of a model in CNF.
///
/// Expects Model to be in the Conjunctive Normal Form:
///
/// - All variables must be boolean
/// - Expressions must be `Reference`, `Not(Reference)`, or `Or(Reference1, Not(Reference2), ...)`
/// - The top level And() may contain nested Or()s. Any other nested expressions are not allowed.
#[derive(Debug, Clone)]
pub struct CNFModel {
    pub CLAUSES: Vec<Vec<i32>>,
    variables: HashMap<conjure_ast::Name, i32>,
    next_ind: i32,
}
impl CNFModel {
    pub fn new() -> CNFModel {
        CNFModel {
            CLAUSES: Vec::new(),
            variables: HashMap::new(),
            next_ind: 1,
        }
    }

    pub fn from_conjure(conjure_model: ConjureModel) -> Result<CNFModel, SolverError> {
        let mut cnf_model: CNFModel = CNFModel::new();

        for var_name in conjure_model.variables.keys() {
            // Check that domain has the correct type
            let decision_var = match conjure_model.variables.get(var_name) {
                None => {
                    return Err(ModelInvalid(format!("variable {:?} not found", var_name)));
                }
                Some(var) => var,
            };

            if decision_var.domain != conjure_ast::Domain::BoolDomain {
                return Err(ModelFeatureNotSupported(format!(
                    "variable {:?} is not BoolDomain",
                    decision_var
                )));
            }

            cnf_model.add_variable(var_name);
        }

        for expr in conjure_model.get_constraints_vec() {
            match cnf_model.add_expression(&expr) {
                Ok(_) => {}
                Err(error) => {
                    let message = format!("{:?}", error);
                    return Err(ModelFeatureNotSupported(message));
                }
            }
        }

        Ok(cnf_model)
    }

    /// Gets all the Conjure variables in the CNF.
    // #[allow(dead_code)] // It will be used once we actually run kissat
    pub fn get_variables(&self) -> Vec<&conjure_ast::Name> {
        let mut ans: Vec<&conjure_ast::Name> = Vec::new();

        for key in self.variables.keys() {
            ans.push(key);
        }

        ans
    }

    /// Gets the index of a Conjure variable.
    pub fn get_index(&self, var: &conjure_ast::Name) -> Option<i32> {
        return self.variables.get(var).copied();
    }

    /// Gets a Conjure variable by index.
    pub fn get_name(&self, ind: i32) -> Option<&conjure_ast::Name> {
        for key in self.variables.keys() {
            let idx = self.get_index(key)?;
            if idx == ind {
                return Some(key);
            }
        }

        None
    }

    /// Adds a new Conjure variable to the CNF representation.
    pub fn add_variable(&mut self, var: &conjure_ast::Name) {
        self.variables.insert(var.clone(), self.next_ind);
        self.next_ind += 1;
    }

    /**
     * Check if a Conjure variable or index is present in the CNF
     */
    pub fn has_variable<T: HasVariable>(&self, value: T) -> bool {
        value.has_variable(self)
    }

    /**
     * Add a new clause to the CNF. Must be a vector of indices in CNF format
     */
    pub fn add_clause(&mut self, vec: &Vec<i32>) -> Result<(), CNFError> {
        for idx in vec {
            if !self.has_variable(idx.abs()) {
                return Err(CNFError::ClauseIndexNotFound(*idx));
            }
        }
        self.CLAUSES.push(vec.clone());
        Ok(())
    }

    /**
     * Add a new Conjure expression to the CNF. Must be a logical expression in CNF form
     */
    pub fn add_expression(&mut self, expr: &conjure_ast::Expression) -> Result<(), CNFError> {
        for row in self.handle_expression(expr)? {
            self.add_clause(&row)?;
        }
        Ok(())
    }

    /**
     * Convert the CNF to a Conjure expression
     */
    #[allow(dead_code)] // It will be used once we actually run kissat
    pub fn as_expression(&self) -> Result<conjure_ast::Expression, CNFError> {
        let mut expr_clauses: Vec<conjure_ast::Expression> = Vec::new();

        for clause in &self.CLAUSES {
            expr_clauses.push(self.clause_to_expression(clause)?);
        }

        Ok(conjure_ast::Expression::And(Metadata::new(), expr_clauses))
    }

    /**
     * Convert a single clause to a Conjure expression
     */
    fn clause_to_expression(&self, clause: &Vec<i32>) -> Result<conjure_ast::Expression, CNFError> {
        let mut ans: Vec<conjure_ast::Expression> = Vec::new();

        for idx in clause {
            match self.get_name(idx.abs()) {
                None => return Err(CNFError::ClauseIndexNotFound(*idx)),
                Some(name) => {
                    if *idx > 0 {
                        ans.push(conjure_ast::Expression::Reference(
                            Metadata::new(),
                            name.clone(),
                        ));
                    } else {
                        let expression: conjure_ast::Expression =
                            conjure_ast::Expression::Reference(Metadata::new(), name.clone());
                        ans.push(conjure_ast::Expression::Not(
                            Metadata::new(),
                            Box::from(expression),
                        ))
                    }
                }
            }
        }

        Ok(conjure_ast::Expression::Or(Metadata::new(), ans))
    }

    /**
     * Get the index for a Conjure Reference or return an error
     * @see get_index
     * @see conjure_ast::Expression::Reference
     */
    fn get_reference_index(&self, name: &conjure_ast::Name) -> Result<i32, CNFError> {
        match self.get_index(name) {
            None => Err(CNFError::VariableNameNotFound(name.clone())),
            Some(ind) => Ok(ind),
        }
    }

    /**
     * Convert the contents of a single Reference to a row of the CNF format
     * @see get_reference_index
     * @see conjure_ast::Expression::Reference
     */
    fn handle_reference(&self, name: &conjure_ast::Name) -> Result<Vec<i32>, CNFError> {
        Ok(vec![self.get_reference_index(name)?])
    }

    /**
     * Convert the contents of a single Not() to CNF
     */
    fn handle_not(&self, expr: &conjure_ast::Expression) -> Result<Vec<i32>, CNFError> {
        match expr {
            // Expression inside the Not()
            conjure_ast::Expression::Reference(_metadata, name) => {
                Ok(vec![-self.get_reference_index(name)?])
            }
            _ => Err(CNFError::UnexpectedExpressionInsideNot(expr.clone())),
        }
    }

    /**
     * Convert the contents of a single Or() to a row of the CNF format
     */
    fn handle_or(&self, expressions: &Vec<conjure_ast::Expression>) -> Result<Vec<i32>, CNFError> {
        let mut ans: Vec<i32> = Vec::new();

        for expr in expressions {
            let ret = self.handle_flat_expression(expr)?;
            for ind in ret {
                ans.push(ind);
            }
        }

        Ok(ans)
    }

    /**
     * Convert a single Reference, `Not` or `Or` into a clause of the CNF format
     */
    fn handle_flat_expression(
        &self,
        expression: &conjure_ast::Expression,
    ) -> Result<Vec<i32>, CNFError> {
        match expression {
            conjure_ast::Expression::Reference(_metadata, name) => self.handle_reference(name),
            conjure_ast::Expression::Not(_metadata, var_box) => self.handle_not(var_box),
            conjure_ast::Expression::Or(_metadata, expressions) => self.handle_or(expressions),
            _ => Err(CNFError::UnexpectedExpression(expression.clone())),
        }
    }

    /**
     * Convert a single And() into a vector of clauses in the CNF format
     */
    fn handle_and(
        &self,
        expressions: &Vec<conjure_ast::Expression>,
    ) -> Result<Vec<Vec<i32>>, CNFError> {
        let mut ans: Vec<Vec<i32>> = Vec::new();

        for expression in expressions {
            match expression {
                conjure_ast::Expression::And(_metadata, _expressions) => {
                    return Err(CNFError::NestedAnd(expression.clone()));
                }
                _ => {
                    ans.push(self.handle_flat_expression(expression)?);
                }
            }
        }

        Ok(ans)
    }

    /**
     * Convert a single Conjure expression into a vector of clauses of the CNF format
     */
    fn handle_expression(
        &self,
        expression: &conjure_ast::Expression,
    ) -> Result<Vec<Vec<i32>>, CNFError> {
        match expression {
            conjure_ast::Expression::And(_metadata, expressions) => self.handle_and(expressions),
            _ => Ok(vec![self.handle_flat_expression(expression)?]),
        }
    }
}

impl Default for CNFModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for checking if a variable is present in the CNF polymorphically (i32 or conjure_ast::Name)
pub trait HasVariable {
    fn has_variable(self, cnf: &CNFModel) -> bool;
}

impl HasVariable for i32 {
    fn has_variable(self, cnf: &CNFModel) -> bool {
        return cnf.get_name(self).is_some();
    }
}

impl HasVariable for &conjure_ast::Name {
    fn has_variable(self, cnf: &CNFModel) -> bool {
        cnf.get_index(self).is_some()
    }
}
