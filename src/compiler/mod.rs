// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

//! Semauri compiler implementation.
//!
//! Source files are grouped by compiler responsibility while remaining in one
//! internal module during the staged modularization. This preserves the public
//! API and existing private helper contracts without a behavior-changing rewrite.

use serde::Serialize;
use serde_json::{json, Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

include!("core/diagnostics.rs");
include!("core/types.rs");
include!("core/text_conversions.rs");
include!("core/unicode_scalars.rs");
include!("core/json_decode.rs");

include!("domains/model.rs");
include!("domains/registry.rs");
include!("domains/builtins/common.rs");
include!("domains/builtins/config.rs");
include!("domains/builtins/ai.rs");
include!("domains/builtins/web.rs");
include!("domains/builtins/structured_data.rs");
include!("domains/builtins/filesystem.rs");
include!("domains/builtins/console.rs");
include!("domains/builtins/ml.rs");

include!("frontend/lexer.rs");
include!("frontend/ast.rs");
include!("frontend/parser.rs");
include!("frontend/formatter.rs");

include!("semantic/hir_model.rs");
include!("semantic/hir_builder.rs");
include!("semantic/optimizer.rs");
include!("semantic/effects.rs");

include!("runtime/model.rs");
include!("runtime/dataflow.rs");
include!("runtime/ml_audit.rs");
include!("runtime/lowering.rs");

include!("backends/api.rs");
include!("backends/compiler.rs");
include!("backends/html.rs");
include!("backends/json_schema.rs");
include!("backends/shell.rs");
include!("backends/filesystem_executor.rs");
include!("backends/runtime_executor.rs");

include!("cli.rs");
include!("backends/test_runner.rs");
include!("tests.rs");
