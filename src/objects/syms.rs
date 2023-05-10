use crate::{ArmergeKeepOrRemove, MergeError};
use object::{Object, ObjectSymbol, SymbolKind};
use rayon::prelude::*;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct ObjectSyms {
    globals: HashSet<String>,
    undefineds: HashSet<String>,
    pub kept_syms_list: String,
    pub has_exported_symbols: bool,
    pub deps: HashSet<PathBuf>,
}

impl ObjectSyms {
    pub fn new(
        object_path: &Path,
        keep_or_remove: ArmergeKeepOrRemove,
        regexes: &[Regex],
    ) -> Result<Self, MergeError> {
        let mut globals = HashSet::new();
        let mut undefineds = HashSet::new();
        let mut kept_syms_list = String::new();
        let mut has_exported_symbols = false;
        let mut kept_obj = false;

        if keep_or_remove == ArmergeKeepOrRemove::KeepObjects {
            let filename_cow = object_path.file_name().unwrap().to_string_lossy();
            let filename: &str = filename_cow.as_ref();
            for regex in regexes {
                if regex.is_match(filename) {
                    kept_obj = true;
                    break;
                }
            }
        }

        let data = std::fs::read(object_path)?;
        let file = object::File::parse(data.as_slice()).map_err(|e| MergeError::InvalidObject {
            path: object_path.to_owned(),
            inner: e,
        })?;
        for sym in file.symbols() {
            if sym.kind() != SymbolKind::Text
                && sym.kind() != SymbolKind::Data
                && sym.kind() != SymbolKind::Unknown
            {
                continue;
            }

            if let Ok(name) = sym.name() {
                if sym.is_undefined() {
                    undefineds.insert(name.to_owned());
                } else if sym.is_global() || sym.is_weak() {
                    globals.insert(name.to_owned());
                }
            }

            if !sym.is_global() || sym.is_undefined() {
                continue;
            }

            if let Ok(name) = sym.name() {
                if keep_or_remove == ArmergeKeepOrRemove::KeepObjects {
                    if kept_obj {
                        has_exported_symbols = true;
                        kept_syms_list += name;
                        kept_syms_list.push('\n');
                    }
                } else {
                    for regex in regexes {
                        let keep_sym_condition = if keep_or_remove == ArmergeKeepOrRemove::KeepSymbols {
                            regex.is_match(name)
                        } else {
                            !regex.is_match(name)
                        };
                        if keep_sym_condition {
                            has_exported_symbols = true;
                            kept_syms_list += name;
                            kept_syms_list.push('\n');
                            break;
                        }
                    }
                }
            }
        }

        Ok(Self {
            globals,
            undefineds,
            has_exported_symbols,
            kept_syms_list,
            deps: Default::default(),
        })
    }

    pub fn has_dependency(&self, obj_syms: &ObjectSyms) -> bool {
        for undef in &self.undefineds {
            if obj_syms.globals.contains(undef) {
                return true;
            }
        }
        false
    }

    pub fn check_dependencies(object_syms: &mut HashMap<PathBuf, Self>) {
        let deps_map = object_syms
            .par_iter()
            .map(|(left_path, left_syms)| {
                let mut deps = HashSet::new();
                for (right_path, right_syms) in object_syms.iter() {
                    if std::ptr::eq(left_path, right_path) {
                        continue;
                    }

                    if left_syms.has_dependency(right_syms) {
                        deps.insert(right_path.to_owned());
                    }
                }
                (left_path.to_owned(), deps)
            })
            .collect::<HashMap<_, _>>();
        for (path, deps) in deps_map {
            object_syms.get_mut(&path).unwrap().deps = deps;
        }
    }
}
