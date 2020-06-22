//!
//! The default evaluation test runner.
//!

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use colored::Colorize;
use serde_json::Map as JsonMap;

use zinc_vm::Bn256;
use zinc_vm::IFacade;

use crate::file::File;
use crate::metadata::Metadata;
use crate::program::Program;
use crate::runners::Runnable;
use crate::Summary;

#[derive(Clone)]
pub struct Runner {
    pub verbosity: usize,
    pub filter: Option<String>,
}

impl Runner {
    pub fn new(verbosity: usize, filter: Option<String>) -> Self {
        Self { verbosity, filter }
    }
}

impl Runnable for Runner {
    fn run(self, path: PathBuf, file: File, metadata: Metadata, summary: Arc<Mutex<Summary>>) {
        let path = match path.strip_prefix(crate::TESTS_DIRECTORY) {
            Ok(path) => path,
            Err(_error) => &path,
        };

        for case in metadata.cases.into_iter() {
            let case_name = format!("{}::{}", path.to_string_lossy(), case.case);
            if let Some(filter) = self.filter.as_ref() {
                if !case_name.contains(filter) {
                    continue;
                }
            }

            if metadata.ignore || case.ignore {
                summary.lock().expect(crate::panic::MUTEX_SYNC).ignored += 1;
                println!("[INTEGRATION] {} {}", "IGNORE".yellow(), case_name);
                continue;
            }

            let program =
                match Program::new(file.code.as_str(), path.to_owned(), case.entry, case.input) {
                    Ok(program) => program,
                    Err(error) => {
                        summary.lock().expect(crate::panic::MUTEX_SYNC).invalid += 1;
                        println!(
                            "[INTEGRATION] {} {} ({})",
                            "INVALID".red(),
                            case_name,
                            error
                        );
                        continue;
                    }
                };

            match program.bytecode.run::<Bn256>(program.witness) {
                Ok(output) => {
                    let output = output
                        .try_into_json()
                        .unwrap_or_else(|| JsonMap::new().into());
                    if case.expect == output {
                        if !case.should_panic {
                            summary.lock().expect(crate::panic::MUTEX_SYNC).passed += 1;
                            if self.verbosity > 0 {
                                println!("[INTEGRATION] {} {}", "PASSED".green(), case_name);
                            }
                        } else {
                            summary.lock().expect(crate::panic::MUTEX_SYNC).failed += 1;
                            println!(
                                "[INTEGRATION] {} {} (should have panicked)",
                                "FAILED".bright_red(),
                                case_name
                            );
                        }
                    } else {
                        summary.lock().expect(crate::panic::MUTEX_SYNC).failed += 1;
                        println!(
                            "[INTEGRATION] {} {} (expected {}, but got {})",
                            "FAILED".bright_red(),
                            case_name,
                            case.expect,
                            output
                        );
                    }
                }
                Err(error) => {
                    if case.should_panic {
                        summary.lock().expect(crate::panic::MUTEX_SYNC).passed += 1;
                        if self.verbosity > 0 {
                            println!(
                                "[INTEGRATION] {} {} (panicked)",
                                "PASSED".green(),
                                case_name
                            );
                        }
                    } else {
                        summary.lock().expect(crate::panic::MUTEX_SYNC).failed += 1;
                        println!(
                            "[INTEGRATION] {} {} ({})",
                            "FAILED".bright_red(),
                            case_name,
                            error
                        );
                    }
                }
            }
        }
    }
}
