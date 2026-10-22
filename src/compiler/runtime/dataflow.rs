// A portable, validated dataflow view of runtime plans. This does not
// execute any operation or authorize any declared effect.
impl RuntimePlan {
    pub fn dataflow_graph(&self) -> Result<JsonValue> {
        let mut nodes = Vec::with_capacity(self.operations.len());
        let mut edges = Vec::new();

        for (index, operation) in self.operations.iter().enumerate() {
            let expected_id = index + 1;
            if operation.id != expected_id {
                return Err(SemauriError::semantic(
                    "S347",
                    format!(
                        "Runtime operation IDs must be consecutive: expected #{expected_id}, got #{}",
                        operation.id
                    ),
                    Some(operation.source_span),
                    None,
                ));
            }

            match (&operation.return_type, &operation.result) {
                (Type::Unit, None) => {}
                (Type::Unit, Some(_)) | (_, None) => {
                    return Err(SemauriError::semantic(
                        "S347",
                        format!("Runtime operation #{} has an inconsistent result", operation.id),
                        Some(operation.source_span),
                        None,
                    ));
                }
                (ty, Some(result)) => {
                    if result.producer_id != operation.id
                        || result.id != format!("%{}", operation.id)
                        || result.ty != *ty
                    {
                        return Err(SemauriError::semantic(
                            "S347",
                            format!("Runtime operation #{} has an invalid result reference", operation.id),
                            Some(operation.source_span),
                            None,
                        ));
                    }
                }
            }

            let mut dependencies = BTreeSet::new();
            for (argument, value) in &operation.arguments {
                let RuntimeArgument::Ref(reference) = value else {
                    continue;
                };
                if reference.producer_id == 0 || reference.producer_id >= operation.id {
                    return Err(SemauriError::semantic(
                        "S347",
                        format!(
                            "Operation #{} argument '{}' references a missing, future, or cyclic producer #{}",
                            operation.id, argument, reference.producer_id
                        ),
                        Some(reference.source_span),
                        None,
                    ));
                }

                let producer = &self.operations[reference.producer_id - 1];
                let Some(produced) = &producer.result else {
                    return Err(SemauriError::semantic(
                        "S347",
                        format!(
                            "Operation #{} argument '{}' references a producer with no value",
                            operation.id, argument
                        ),
                        Some(reference.source_span),
                        None,
                    ));
                };
                if produced.id != reference.id
                    || Type::assignment_kind(&produced.ty, &reference.ty).is_none()
                {
                    return Err(SemauriError::semantic(
                        "S347",
                        format!(
                            "Operation #{} argument '{}' has a mismatched producer ID or type",
                            operation.id, argument
                        ),
                        Some(reference.source_span),
                        None,
                    ));
                }
                dependencies.insert(reference.producer_id);
                edges.push(json!({
                    "producer": reference.producer_id,
                    "consumer": operation.id,
                    "argument": argument,
                    "value_ref": reference.id,
                    "type": reference.ty.to_json(),
                }));
            }

            nodes.push(json!({
                "id": operation.id,
                "domain": operation.domain,
                "name": operation.name,
                "depends_on": dependencies.into_iter().collect::<Vec<_>>(),
                "arguments": operation.arguments.iter().map(|(key, value)| (key.clone(), value.to_json())).collect::<JsonMap<_, _>>(),
                "produces": operation.result.as_ref().map(RuntimeValueRef::to_json),
                "effects": operation.effects,
                "source_span": operation.source_span,
            }));
        }

        Ok(json!({
            "kind": "runtime_dataflow_graph",
            "schema_version": 1,
            "operations": nodes,
            "edges": edges,
        }))
    }
}
