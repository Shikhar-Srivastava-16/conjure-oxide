use anyhow::Error;
use rustsat::solvers::SolverResult;
use sat_rs::sat_solvers::SatSolver;
use sat_rs::solver_utils::{initialize_solver, solve_problem};

// fn main() -> Result<(), Error> {
fn main() -> () {
    let v1: Vec<i32> = vec![1, 2, -3];
    let v2: Vec<i32> = vec![-1, 3];
    let v3: Vec<i32> = vec![2, -3];
    let v4: Vec<i32> = vec![-2, 3];
    let v5: Vec<i32> = vec![1, -2];

    let vec_problem: Vec<Vec<i32>> = vec![v1, v2, v3, v4, v5];

    // Initializes SAT Solver and SAT Instance
    // let (sat_solver, inst) = initialize_solver(&vec_problem);

    // Solves the Problem
    // let res: Result<SolverResult, Error> = solve_problem(&sat_solver, inst);
    // let res: Result<SolverResult> = solve_problem(&sat_solver, inst)?;

    // Solver Result
    // match res {
    //     SolverResult::Sat => {
    //         println!("SATISFIABLE");
    //     }
    //     SolverResult::Unsat => println!("UNSATISFIABLE"),
    // }
    // Ok(())

    // match res {
    //     Ok
    // }
}

// fn main() {
//     /*
//      * Problem: (t or t) and (f or t) and (f or f)
//      */
//     let v1: Vec<i16> = vec![1, 1];
//     let v2: Vec<i16> = vec![0, 1];
//     let v3: Vec<i16> = vec![0, 0];

//     let vec_problem: Vec<Vec<i16>> = vec![v1, v2, v3];

//     // tree (?)
//     // for load_model() (?) use inst(?)
//     // TODO consult rest
//     let mut inst: SatInstance = SatInstance::new();
//     sat_tree::conv_to_formula(&vec_problem, &mut inst);

//     // sat solver
//     // for solve(), convert res to fit Result<SolveSuccess, SolverError>
//     // convert SoverResult to SolveSuccess (?) in sat_tree or sat_solvers
//     let cnf_func = inst.into_cnf();
//     let mut solver = Minisat::default();
//     solver.add_cnf(cnf_func.0);

//     let res: Result<SolverResult, Error> = solver.solve();
//     let unwrap: SolverResult = res.unwrap();

//     // solve:
//     // match solver.solve() {
//     //     Ok(true) => {
//     //         // If satisfiable, print satisfying assignments
//     //         println!("SATISFIABLE");
//     //         // println!("a = {}", solver.value(var_a).unwrap());
//     //         // println!("b = {}", solver.value(var_b).unwrap());
//     //     }
//     //     Ok(false) => println!("UNSATISFIABLE"),
//     //     Err(e) => println!("Error during solving: {:?}", e),
//     // }

// // or perhaps
// solver.solve();
// }

// // #[cfg(test)]
// // #[test]
// // fn test1() {
// //     use kissat_rs::Assignment;
// //     use kissat_rs::Solver;

// //     // Define three literals used in both formulae.
// //     let x = 1;
// //     let y = 2;
// //     let z = 3;

// //     // Construct a formula from clauses (i.e. an iterator over literals).
// //     // (~x || y) && (~y || z) && (x || ~z) && (x || y || z)
// //     let formula1 = vec![vec![-x, y], vec![-y, z], vec![x, -z], vec![x, y, z]];
// //     let satisfying_assignment = Solver::solve_formula(formula1).unwrap();

// //     // The formula from above is satisfied by the assignment: x -> True, y -> True, z -> True
// //     if let Some(assignments) = satisfying_assignment {
// //         assert_eq!(assignments.get(&x).unwrap(), &Some(Assignment::True));
// //         assert_eq!(assignments.get(&y).unwrap(), &Some(Assignment::True));
// //         assert_eq!(assignments.get(&z).unwrap(), &Some(Assignment::True));
// //     }

// //     // (x || y || ~z) && ~x && (x || y || z) && (x || ~y)
// //     let formula2 = vec![vec![x, y, -z], vec![-x], vec![x, y, z], vec![x, -y]];
// //     let unsat_result = Solver::solve_formula(formula2).unwrap();

// //     // The second formula is unsatisfiable.
// //     // This can for example be proved by resolution.
// //     assert_eq!(unsat_result, None);
// // }
