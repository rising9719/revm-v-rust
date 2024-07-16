mod test_suite;

pub use test_suite::{PragueTestResult, TestResult, TestSuite, TestUnit, TestVector};

use crate::{cmd::Error, dir_utils::find_all_json_tests};
use revm::{
    interpreter::analysis::{validate_raw_eof, EofError},
    primitives::Eof,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

/// Eof validation command.
#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// Input path to eof validation test
    #[structopt(required = true)]
    path: PathBuf,
}

impl Cmd {
    /// Run statetest command.
    pub fn run(&self) -> Result<(), Error> {
        // check if path exists.
        if !self.path.exists() {
            return Err(Error::Custom("The specified path does not exist"));
        }
        run_test(&self.path);
        Ok(())
    }
}

pub fn run_test(path: &Path) {
    let test_files = find_all_json_tests(path);
    let mut test_sum = 0;
    let mut passed_tests = 0;

    #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
    enum ErrorType {
        FalsePositive,
        Error(EofError),
    }
    let mut types_of_error: BTreeMap<ErrorType, usize> = BTreeMap::new();
    for test_file in test_files {
        let s = std::fs::read_to_string(test_file).unwrap();
        let suite: TestSuite = serde_json::from_str(&s).unwrap();
        for (name, test_unit) in suite.0 {
            for (vector_name, test_vector) in test_unit.vectors {
                test_sum += 1;
                let res = validate_raw_eof(test_vector.code.clone());
                if res.is_ok() != test_vector.results.prague.result {
                    let eof = Eof::decode(test_vector.code.clone());
                    println!(
                        "\nTest failed: {} - {}\nresult:{:?}\nrevm err_result:{:#?}\nbytes:{:?}\n,eof:{eof:#?}",
                        name,
                        vector_name,
                        test_vector.results.prague,
                        res.as_ref().err(),
                        test_vector.code
                    );
                    *types_of_error
                        .entry(
                            res.err()
                                .map(ErrorType::Error)
                                .unwrap_or(ErrorType::FalsePositive),
                        )
                        .or_default() += 1;
                } else {
                    passed_tests += 1;
                }
            }
        }
    }
    println!("Types of error: {:#?}", types_of_error);
    println!("Passed tests: {}/{}", passed_tests, test_sum);
}
