use std::any::type_name;
use std::fmt::format;
use std::iter::Inspect;
use std::ptr::null;
use std::vec;

use clap::error;
use minion_rs::ast::Model;
use rustsat::encodings::am1::Def;
use rustsat::solvers::{Solve, SolverResult};
use rustsat::types::Var as satVar;
use sat_rs::sat_tree::{self, conv_to_clause, conv_to_formula};
use std::collections::HashMap;

use rustsat_minisat::core::Minisat;

use crate::ast::{Expression, Name};
use crate::metadata::Metadata;
use crate::solver::{self, SearchComplete, SearchStatus, SolveSuccess, SolverCallback, SolverFamily, SolverMutCallback};
use crate::stats::SolverStats;
use crate::{ast as conjure_ast, model, Model as ConjureModel};

use super::super::model_modifier::NotModifiable;
use super::super::private;
use super::super::SearchComplete::*;
use super::super::SearchIncomplete::*;
use super::super::SearchStatus::*;
use super::super::SolverAdaptor;
use super::super::SolverError;
use super::super::SolverError::*;
use super::super::SolverError::*;

use rustsat::instances::SatInstance;

use thiserror::Error;

/// A [SolverAdaptor] for interacting with the SatSolver generic and the types thereof.

pub struct SAT {
    __non_constructable: private::Internal,
    model_inst: Option<SatInstance>,
    var_map: Option<HashMap<i32, satVar>>,
    solver_inst: Minisat,
}

impl private::Sealed for SAT {}

impl Default for SAT {
    fn default() -> Self {
        SAT {
            __non_constructable: private::Internal,
            model_inst: None,
            var_map: None,
            solver_inst: Minisat::default(),
        }
    }
}

impl SAT {
    pub fn new(model: ConjureModel) -> Self {
        let model_to_use: Option<SatInstance> = Some(SatInstance::new());
        SAT {
            __non_constructable: private::Internal,
            model_inst: model_to_use,
            var_map: None,
            solver_inst: Minisat::default(),
        }
    }

    pub fn add_clause_to_mod(&self, clause_vec: Vec<i32>) -> () {}
}

pub fn instantiate_model_from_conjure(
    conjure_model: ConjureModel,
) -> Result<SatInstance, SolverError> {
    let mut inst: SatInstance = SatInstance::new();

    for var_name_ref in conjure_model.variables.keys() {
        let curr_decision_var = conjure_model
            .variables
            .get(var_name_ref)
            .ok_or_else(|| ModelInvalid(format!("variable {:?} not found", var_name_ref)))?;

        {
            // todo: the scope change may be unneeded
            // check domain, err if bad domain
            let cdom = &curr_decision_var.domain;
            if cdom != &conjure_ast::Domain::BoolDomain {
                return Err(ModelFeatureNotSupported(format!(
                    "variable {:?}: expected BoolDomain, found: {:?}",
                    curr_decision_var, curr_decision_var.domain
                )));
            }
        }
    }

    let md = Metadata {
        clean: false,
        etype: None,
    };

    let constraints_vec: Vec<Expression> = conjure_model.get_constraints_vec();
    let vec_cnf = handle_and(Expression::And(md, constraints_vec));
    conv_to_formula(&(vec_cnf.unwrap()), &mut inst);

    Ok(inst)
}

impl SolverAdaptor for SAT {
    fn solve(
        &mut self,
        callback: SolverCallback,
        _: private::Internal,
    ) -> Result<SolveSuccess, SolverError> {

        let cnf_func: rustsat::instances::Cnf = self.model_inst.clone().unwrap().into_cnf().0;
        &self.solver_inst.add_cnf(cnf_func);
        let res = &self.solver_inst.solve().unwrap();

        let solver_res = match res {
            SolverResult::Sat => true,
            SolverResult::Unsat => false,

            // should not arise:
            SolverResult::Interrupted => Err(SolverError::Runtime(format!("SatInstance may be invalid, Interrupted.")))?,
        };

        // will eventually have a SolveSucess instance being returned with Ok(), when the implementation is more permanent.
        // let solve_success = SolveSuccess{
        //     // arbitrary time
        //     stats:SolverStats.with_timings(10.0),
        //     // sat solvers only produce 1 sol?
        //     status: SearchStatus::Complete((
        //         // should be correct, recheck
        //         SearchComplete::HasSolutions
        //     ))
        // };
        print!("{}", solver_res);

        // Err(OpNotImplemented("solve_mut".to_owned()))
        Ok(())
    }

    fn solve_mut(
        &mut self,
        callback: SolverMutCallback,
        _: private::Internal,
    ) -> Result<SolveSuccess, SolverError> {
        Err(OpNotSupported("solve_mut".to_owned()))
    }

    fn load_model(&mut self, model: ConjureModel, _: private::Internal) -> Result<(), SolverError> {
        let inst_res = instantiate_model_from_conjure(model);
        self.model_inst = Some(inst_res.unwrap());
        Ok(())
    }

    fn get_family(&self) -> SolverFamily {
        SolverFamily::SAT
    }
}

pub fn get_namevar_as_int(name: Name) -> Result<i32, CNFError> {
    match name {
        Name::MachineName(val) => Ok(val),
        _ => Err(CNFError::BadVariableType(name)),
    }
}

pub fn handle_lit(e: Expression) -> Result<i32, CNFError> {
    match e {
        Expression::Not(_, heap_expr) => {
            let expr = *heap_expr;
            match expr {
                Expression::Not(_md, e) => handle_lit(*e),
                Expression::Reference(_md, name) => {
                    let check = get_namevar_as_int(name).unwrap();
                    match check == 0 {
                        true => Ok(1),
                        false => Ok(0),
                    }
                }
                _ => Err(CNFError::UnexpectedExpressionInsideNot(expr)),
            }
        }
        Expression::Reference(_md, name) => get_namevar_as_int(name),
        _ => Err(CNFError::UnexpectedLiteralExpression(e)),
    }
}

pub fn handle_or(e: Expression) -> Result<(Vec<i32>), CNFError> {
    let vec_clause = match e {
        Expression::Or(_md, vec) => vec,
        _ => Err(CNFError::UnexpectedExpression(e))?,
    };

    let mut ret_clause: Vec<i32> = Vec::new();

    for expr in vec_clause {
        match expr {
            Expression::Reference(_, _) => ret_clause.push(handle_lit(expr).unwrap()),
            Expression::Not(_, _) => ret_clause.push(handle_lit(expr).unwrap()),
            _ => Err(CNFError::UnexpectedExpressionInsideOr(expr))?,
        }
    }

    Ok(ret_clause)
}

pub fn handle_and(e: Expression) -> Result<(Vec<Vec<i32>>), CNFError> {
    let vec_cnf = match e {
        Expression::And(_md, vec_and) => vec_and,
        _ => panic!("Villain, What hast thou done?\nThat which thou canst not undo."),
    };

    let mut ret_vec_of_vecs: Vec<Vec<i32>> = Vec::new();

    for expr in vec_cnf {
        match expr {
            Expression::Or(_, _) => ret_vec_of_vecs.push(handle_or(expr).unwrap()),
            _ => Err(CNFError::UnexpectedExpressionInsideOr(expr))?,
        }
    }

    Ok(ret_vec_of_vecs)
}

#[derive(Error, Debug)]
pub enum CNFError {
    #[error("Variable with name `{0}` not found")]
    VariableNameNotFound(conjure_ast::Name),

    #[error("Variable with name `{0}` not of right type")]
    BadVariableType(Name),

    #[error("Unexpected Expression `{0}` inside Not(). Only Not(Reference) or Not(Not) allowed!")]
    UnexpectedExpressionInsideNot(Expression),

    #[error("Unexpected Expression `{0}` as literal. Only Not() or Reference() allowed!")]
    UnexpectedLiteralExpression(Expression),

    #[error("Unexpected Expression `{0}` inside And(). Only And(vec<Or>) allowed!")]
    UnexpectedExpressionInsideAnd(Expression),

    #[error("Unexpected Expression `{0}` inside Or(). Only Or(lit, lit) allowed!")]
    UnexpectedExpressionInsideOr(Expression),

    #[error("Unexpected Expression `{0}` found!")]
    UnexpectedExpression(Expression)
}


#[cfg(test)]
mod tests {

    use conjure_ast::ReturnType;
    use super::*;

    #[test]
    fn test_handle_and() {

    }

    #[test]
    fn test_handle_or() {
        let lit_1 = Expression::Reference(
            Metadata(
                true,
                Some(ReturnType::Bool)
            ), 
            Name::MachineName((1))
        );

        let lit_for_not = Expression::Reference(
            Metadata(
                true,
                Some(ReturnType::Bool)
            ), 
            Name::MachineName((1))
        );

        let not_lit = Expression::Not(
            Metadata(
                true,
                Some(ReturnType::Bool)
            ),
            lit_for_not
        );
        
        let expr_or = Expression::Or(
            Metadata(
                true,
                Some(ReturnType::Bool)
            ), 
            vec![lit_1, not_lit]
        );
        
        assert_eq!(handle_or(lit_1).unwrap(), 1);
    }
     
    #[test]
    fn test_handle_lit() {
        let lit_1 = Expression::Reference(
            Metadata(
                true,
                Some(ReturnType::Bool)
            ), 
            Name::MachineName((1))
        );
        
        
        let lit_2 = Expression::Reference(
            Metadata(
                true,
                Some(ReturnType::Bool)
            ), 
            Name::MachineName((0))
        );

        let lit_for_not = Expression::Reference(
            Metadata(
                true,
                Some(ReturnType::Bool)
            ), 
            Name::MachineName((1))
        );

        let not_lit = Expression::Not(
            Metadata(
                true,
                Some(ReturnType::Bool)
            ),
            lit_for_not
        );
        
        assert_eq!(handle_lit(lit_1).unwrap(), 1);
        assert_eq!(handle_lit(not_lit).unwrap(), 0);
        assert_eq!(handle_lit(lit_2).unwrap(), 0);
    }   
}