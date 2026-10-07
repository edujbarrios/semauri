// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

//! Internal compiler engine.
//!
//! The crate root is a small facade. These concern-oriented source slices are
//! included into one internal module so private helpers keep their existing
//! visibility while the implementation remains physically modular.

use serde::Serialize;
use serde_json::{json, Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

include!("core.rs");
include!("domain/mod.rs");
include!("frontend/lexer.rs");
include!("frontend/ast.rs");
include!("frontend/parser.rs");
include!("compiler/hir.rs");
include!("compiler/optimizer.rs");
include!("compiler/effects.rs");
include!("runtime/ir.rs");
include!("backend/mod.rs");
include!("cli.rs");
include!("tests.rs");
