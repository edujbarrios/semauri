// A conservative, inspectable preflight report for ML experiment plans.
//
// Warning-free means that specific declarations are present, NOT that an
// experiment is reproducible: dataset/model revisions are not resolved or
// verified by Semauri, and no model is executed by this compiler.
impl RuntimePlan {
    pub fn ml_reproducibility_audit(&self) -> Result<JsonValue> {
        // Validate the underlying SSA graph before trusting references.
        self.dataflow_graph()?;

        let mut warnings = Vec::new();
        let mut inputs = Vec::new();
        let mut ml_operations = 0usize;

        for operation in &self.operations {
            if operation.domain != "ml" {
                continue;
            }
            ml_operations += 1;
            let warning = match operation.name.as_str() {
                "open_dataset" => Some((
                    "ML001",
                    "Dataset source has no declared revision; use 'Source dataset ... revision ...' for explicit provenance.",
                )),
                "load_model" => Some((
                    "ML002",
                    "Model identifier has no declared revision; use 'Resolve model ... revision ...' for explicit provenance.",
                )),
                "train" => Some((
                    "ML003",
                    "Direct Train specifies epochs but no explicit training seed; use a seeded training config with Fit.",
                )),
                _ => None,
            };
            if let Some((code, message)) = warning {
                warnings.push(json!({
                    "code": code,
                    "message": message,
                    "operation_id": operation.id,
                    "operation": operation.name,
                    "source_span": operation.source_span,
                }));
            }

            let kind = match operation.name.as_str() {
                "source_dataset_revision" => Some("dataset"),
                "resolve_model_revision" => Some("model"),
                _ => None,
            };
            if let Some(kind) = kind {
                let revision = match operation.arguments.get("revision") {
                    Some(RuntimeArgument::Value(ValueData::String(value))) => value.as_str(),
                    _ => {
                        return Err(SemauriError::semantic(
                            "S349",
                            format!(
                                "Versioned ML input operation #{} has no literal revision",
                                operation.id
                            ),
                            Some(operation.source_span),
                            None,
                        ));
                    }
                };
                inputs.push(json!({
                    "kind": kind,
                    "operation_id": operation.id,
                    "declared_revision": revision,
                    "verified": false,
                    "source_span": operation.source_span,
                }));
            }
        }

        if ml_operations == 0 {
            return Err(SemauriError::semantic(
                "S349",
                "ML reproducibility audit requires at least one ML runtime operation",
                None,
                Some("Audit a source file containing a 'Within ml:' workflow.".to_string()),
            ));
        }

        let warning_count = warnings.len();
        Ok(json!({
            "kind": "ml_reproducibility_audit",
            "schema_version": 1,
            "status": if warning_count == 0 { "declarations_complete" } else { "review_required" },
            "verification": "declarations_only",
            "warning_count": warning_count,
            "warnings": warnings,
            "declared_inputs": inputs,
            "ml_operation_count": ml_operations,
        }))
    }
}
