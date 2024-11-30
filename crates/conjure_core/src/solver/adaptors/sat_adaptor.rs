

// use std::any::type_name;
// use std::fmt::format;
// use std::iter::Inspect;
// use std::ptr::null;
// use std::vec;

// use clap::error;
// use minion_rs::ast::Model;
// use rustsat::encodings::am1::Def;
// use rustsat::solvers::{Solve, SolverResult};
// use rustsat::types::Var as satVar;
// use sat_rs::sat_tree::{self, conv_to_clause, conv_to_formula};
// use std::collections::HashMap;

// use rustsat_minisat::core::Minisat;

// use crate::ast::{Expression, Name};
// use crate::metadata::Metadata;
// use crate::solver::{self, SolveSuccess, SolverCallback, SolverFamily, SolverMutCallback};
// use crate::{ast as conjure_ast, model, Model as ConjureModel};

// use super::super::model_modifier::NotModifiable;
// use super::super::private;
// use super::super::SearchComplete::*;
// use super::super::SearchIncomplete::*;
// use super::super::SearchStatus::*;
// use super::super::SolverAdaptor;
// use super::super::SolverError;
// use super::super::SolverError::*;
// use super::super::SolverError::*;

// use rustsat::instances::SatInstance;

// use thiserror::Error;

// /// A [SolverAdaptor] for interacting with the SatSolver generic and the types thereof.

// pub struct SAT {
//     __non_constructable: private::Internal,
//     model_inst: Option<SatInstance>,
//     var_map: Option<HashMap<i32, satVar>>,
//     solver_inst: Option<Minisat>,
// }

// impl private::Sealed for SAT {}

// impl Default for SAT {
//     fn default() -> Self {
//         SAT {
//             __non_constructable: private::Internal,
//             model_inst: None,
//             var_map: None,
//             solver_inst: Some(Minisat::default()),
//         }
//     }
// }

// impl SAT {
//     pub fn new(model: ConjureModel) -> Self {
//         let model_to_use: Option<SatInstance> = Some(SatInstance::new());
//         SAT {
//             __non_constructable: private::Internal,
//             model_inst: model_to_use,
//             var_map: None,
//             solver_inst: Some(Minisat::default()),
//         }
//     }

//     pub fn add_clause_to_mod(&self, clause_vec: Vec<i32>) -> () {}
// }

// pub fn instantiate_model_from_conjure(
//     conjure_model: ConjureModel,
// ) -> Result<SatInstance, SolverError> {
//     let mut inst: SatInstance = SatInstance::new();

//     for var_name_ref in conjure_model.variables.keys() {
//         let curr_decision_var = conjure_model
//             .variables
//             .get(var_name_ref)
//             .ok_or_else(|| ModelInvalid(format!("variable {:?} not found", var_name_ref)))?;

//         {
//             // todo: the scope change may be unneeded
//             // check domain, err if bad domain
//             let cdom = &curr_decision_var.domain;
//             if cdom != &conjure_ast::Domain::BoolDomain {
//                 return Err(ModelFeatureNotSupported(format!(
//                     "variable {:?}: expected BoolDomain, found: {:?}",
//                     curr_decision_var, curr_decision_var.domain
//                 )));
//             }
//         }
//     }

//     let md = Metadata {
//         clean: false,
//         etype: None,
//     };

//     let constraints_vec: Vec<Expression> = conjure_model.get_constraints_vec();
//     let vec_cnf = handle_and(Expression::And(md, constraints_vec));
//     conv_to_formula(&(vec_cnf.unwrap()), &mut inst);

//     Ok(inst)
// }

// impl SolverAdaptor for SAT {
//     fn solve(
//         &mut self,
//         callback: SolverCallback,
//         _: private::Internal,
//     ) -> Result<SolveSuccess, SolverError> {
//         // solver = self.solver
//         // handle
//         let cnf_func = self.model_inst.clone().unwrap().into_cnf();
//         // let res = self.solver.clone().unwrap().add_cnf(cnf_func.0);
//         Err(OpNotImplemented("solve_mut".to_owned()))
//     }

//     fn solve_mut(
//         &mut self,
//         callback: SolverMutCallback,
//         _: private::Internal,
//     ) -> Result<SolveSuccess, SolverError> {
//         Err(OpNotSupported("solve_mut".to_owned()))
//     }

//     fn load_model(&mut self, model: ConjureModel, _: private::Internal) -> Result<(), SolverError> {
//         let inst_res = instantiate_model_from_conjure(model);
//         self.model_inst = Some(inst_res.unwrap());
//         Ok(())
//     }

//     fn get_family(&self) -> SolverFamily {
//         SolverFamily::SAT
//     }
// }

// pub fn handle_expr(e: Expression) -> Result<(Vec<Vec<i32>>), CNFError> {
//     match e {
//         Expression::And(_, _) => Ok(handle_and(e).unwrap()),
//         _ => Err(CNFError::UnexpectedExpression(e)),
//     }
// }

// pub fn get_namevar_as_int(name: Name) -> Result<i32, CNFError> {
//     match name {
//         Name::MachineName(val) => Ok(val),
//         _ => Err(CNFError::BadVariableType(name)),
//     }
// }

// pub fn handle_lit(e: Expression) -> Result<i32, CNFError> {
//     match e {
//         Expression::Not(_, heap_expr) => {
//             let expr = *heap_expr;
//             match expr {
//                 Expression::Nothing => todo!(), // panic?
//                 Expression::Not(_md, e) => handle_lit(*e),
//                 // todo(ss504): decide
//                 Expression::Reference(_md, name) => {
//                     let check = get_namevar_as_int(name).unwrap();
//                     match check == 0 {
//                         true => Ok(1),
//                         false => Ok(0),
//                     }
//                 }
//                 _ => Err(CNFError::UnexpectedExpressionInsideNot(expr)),
//             }
//         }
//         Expression::Reference(_md, name) => get_namevar_as_int(name),
//         _ => Err(CNFError::UnexpectedLiteralExpression(e)),
//     }
// }

// pub fn handle_or(e: Expression) -> Result<(Vec<i32>), CNFError> {
//     let vec_clause = match e {
//         Expression::Or(_md, vec) => vec,
//         _ => Err(CNFError::UnexpectedExpression(e))?,
//     };

//     // if vec_clause.len() != 2 {
//     //     panic!("Villain, What hast thou done?\nThat which thou canst not undo.")
//     // };

//     let mut ret_clause: Vec<i32> = Vec::new();

//     for expr in vec_clause {
//         match expr {
//             Expression::Reference(_, _) => ret_clause.push(handle_lit(expr).unwrap()),
//             Expression::Not(_, _) => ret_clause.push(handle_lit(expr).unwrap()),
//             _ => Err(CNFError::UnexpectedExpressionInsideOr(expr))?,
//         }
//     }

//     Ok(ret_clause)
// }

// pub fn handle_and(e: Expression) -> Result<(Vec<Vec<i32>>), CNFError> {
//     let vec_cnf = match e {
//         Expression::And(_md, vec_and) => vec_and,
//         _ => panic!("Villain, What hast thou done?\nThat which thou canst not undo."),
//     };

//     let mut ret_vec_of_vecs: Vec<Vec<i32>> = Vec::new();

//     for expr in vec_cnf {
//         match expr {
//             Expression::Or(_, _) => ret_vec_of_vecs.push(handle_or(expr).unwrap()),
//             _ => Err(CNFError::UnexpectedExpressionInsideOr(expr))?,
//         }
//     }

//     Ok(ret_vec_of_vecs)
// }
// //CNF Error, may be replaced of integrated with error file
// #[derive(Error, Debug)]
// pub enum CNFError {
//     #[error("Variable with name `{0}` not found")]
//     VariableNameNotFound(conjure_ast::Name),

//     #[error("Variable with name `{0}` not of right type")]
//     BadVariableType(Name),

//     #[error("Unexpected Expression `{0}` inside Not(). Only Not(Reference) or Not(Not) allowed!")]
//     UnexpectedExpressionInsideNot(Expression),

//     #[error("Unexpected Expression `{0}` as literal. Only Not() or Reference() allowed!")]
//     UnexpectedLiteralExpression(Expression),

//     #[error("Unexpected Expression `{0}` inside And(). Only And(vec<Or>) allowed!")]
//     UnexpectedExpressionInsideAnd(Expression),

//     #[error("Unexpected Expression `{0}` inside Or(). Only Or( ) allowed!")]
//     UnexpectedExpressionInsideOr(Expression),

//     #[error("Unexpected Expression `{0}` found!")]
//     UnexpectedExpression(Expression)
// }


use std::collections::HashMap;
use rustsat::solvers::{Solver, SolverResult};
use rustsat::types::{Clause, Lit, Var};
use rustsat_minisat::core::Minisat;
use thiserror::Error;

use crate::ast::{Expression, Name};
use crate::metadata::Metadata;
use crate::solver::{SolverAdaptor, SolveSuccess, SolverCallback, SolverFamily, SolverMutCallback};
use crate::ast as conjure_ast;
use crate::model::{self, Model as ConjureModel};
use crate::solver::SolverError;
use crate::solver::SolverError::*;

#[derive(Default)]
pub struct SAT {
    model_inst: Option<rustsat::instances::SatInstance>,
    var_map: HashMap<i32, Var>,
    solver_inst: Minisat,
}

impl SAT {
    pub fn new(model: ConjureModel) -> Self {
        let mut sat = SAT {
            model_inst: None,
            var_map: HashMap::new(),
            solver_inst: Minisat::new(),
        };

        match instantiate_model_from_conjure(model) {
            Ok(inst) => {
                sat.model_inst = Some(inst.clone());

                for (&var_id, _) in &inst.variables {
                    let sat_var = Var::new(var_id as usize);
                    sat.var_map.insert(var_id, sat_var);
                }

                if let Some(cnf) = inst.into_cnf() {
                    for clause in cnf.clauses {
                        let minisat_clause: Vec<Lit> = clause
                            .iter()
                            .map(|&lit| {
                                let var_id = lit.abs();
                                let minisat_var = sat.var_map.get(&var_id).copied().unwrap();
                                if lit < 0 {
                                    Lit::new(minisat_var, true) // true for negation
                                } else {
                                    Lit::new(minisat_var, false)
                                }
                            })
                            .collect();

                        sat.solver_inst.add_clause(Clause::from(minisat_clause)).unwrap();
                    }
                }
            }
            Err(err) => panic!("Failed to instantiate model: {:?}", err),
        }

        sat
    }

    pub fn add_clause_to_mod(&mut self, clause_vec: Vec<i32>) -> Result<(), SolverError> {
        let minisat_clause: Vec<Lit> = clause_vec
            .iter()
            .map(|&lit| {
                let var_id = lit.abs();
                let sat_var = self.var_map.get(&var_id).copied().ok_or_else(|| {
                    SolverError::VariableNameNotFound(Name::MachineName(var_id))
                })?;
                Ok(if lit < 0 {
                    Lit::new(sat_var, true)
                } else {
                    Lit::new(sat_var, false)
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.solver_inst.add_clause(Clause::from(minisat_clause)).unwrap();
        Ok(())
    }

    pub fn solve(&mut self, callback: SolverCallback) -> Result<SolveSuccess, SolverError> {
        match self.solver_inst.solve() {
            Ok(SolverResult::Sat) => {
                let model = self
                    .solver_inst
                    .model()
                    .ok_or_else(|| SolverError::ModelInvalid("Solver did not return a model".to_string()))?;

                let mut assignment = HashMap::new();
                for (&var_id, &sat_var) in &self.var_map {
                    let value = model.value(sat_var).ok_or_else(|| {
                        SolverError::ModelInvalid(format!("Value not found for variable {:?}", sat_var))
                    })?;
                    assignment.insert(var_id, value);
                }

                callback(&assignment);
                Ok(SolveSuccess::Satisfiable)
            }
            Ok(SolverResult::Unsat) => Ok(SolveSuccess::Unsatisfiable),
            Err(err) => Err(SolverError::SolverTimeout(format!("{:?}", err))),
        }
    }
}

pub fn instantiate_model_from_conjure(
    conjure_model: ConjureModel,
) -> Result<rustsat::instances::SatInstance, SolverError> {
    let mut inst = rustsat::instances::SatInstance::new();

    for var_name_ref in conjure_model.variables.keys() {
        let curr_decision_var = conjure_model
            .variables
            .get(var_name_ref)
            .ok_or_else(|| ModelInvalid(format!("Variable {:?} not found", var_name_ref)))?;

        if curr_decision_var.domain != conjure_ast::Domain::BoolDomain {
            return Err(ModelFeatureNotSupported(format!(
                "Variable {:?} has unsupported domain: {:?}",
                curr_decision_var, curr_decision_var.domain
            )));
        }
    }

    let md = Metadata {
        clean: false,
        etype: None,
    };

    let constraints_vec = conjure_model.get_constraints_vec();
    let vec_cnf = handle_and(Expression::And(md, constraints_vec))?;
    conv_to_formula(&vec_cnf, &mut inst);

    Ok(inst)
}

pub fn handle_and(e: Expression) -> Result<Vec<Vec<i32>>, CNFError> {
    match e {
        Expression::And(_, vec_and) => {
            let mut clauses = Vec::new();
            for expr in vec_and {
                clauses.push(handle_or(expr)?);
            }
            Ok(clauses)
        }
        _ => Err(CNFError::UnexpectedExpression(e)),
    }
}

pub fn handle_or(e: Expression) -> Result<Vec<i32>, CNFError> {
    match e {
        Expression::Or(_, vec_or) => {
            let mut clause = Vec::new();
            for expr in vec_or {
                match expr {
                    Expression::Reference(_, _) | Expression::Not(_, _) => {
                        clause.push(handle_lit(expr)?);
                    }
                    _ => return Err(CNFError::UnexpectedExpressionInsideOr(expr)),
                }
            }
            Ok(clause)
        }
        _ => Err(CNFError::UnexpectedExpression(e)),
    }
}

pub fn handle_lit(e: Expression) -> Result<i32, CNFError> {
    match e {
        Expression::Reference(_, name) => get_namevar_as_int(name),
        Expression::Not(_, box_expr) => Ok(-handle_lit(*box_expr)?),
        _ => Err(CNFError::UnexpectedLiteralExpression(e)),
    }
}

pub fn get_namevar_as_int(name: Name) -> Result<i32, CNFError> {
    match name {
        Name::MachineName(val) => Ok(val),
        _ => Err(CNFError::BadVariableType(name)),
    }
}
