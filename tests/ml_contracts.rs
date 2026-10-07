use semauri::Compiler;

fn ml_prefix() -> &'static str {
    "Within ml: Let dataset be Open dataset \"./data\". Let model be Load model \"resnet18\". Let device be Select device \"cpu\". "
}

#[test]
fn ml_dataset_split_and_training_validation_codes_are_preserved() {
    let compiler = Compiler::new();

    let split = format!(
        "{}Let partition be Split dataset ratio 1.0 seed 1. End.",
        ml_prefix()
    );
    assert_eq!(compiler.hir(&split).unwrap_err().code, "S340");

    let optimizer = format!(
        "{}Let config be Configure training for 2 epochs using optimizer \"rmsprop\" learning rate 0.001 batch size 8 seed 1. End.",
        ml_prefix()
    );
    assert_eq!(compiler.hir(&optimizer).unwrap_err().code, "S336");

    let numeric = format!(
        "{}Let config be Configure training for 0 epochs using optimizer \"adam\" learning rate 0.001 batch size 8 seed 1. End.",
        ml_prefix()
    );
    assert_eq!(compiler.hir(&numeric).unwrap_err().code, "S336");
}

#[test]
fn ml_model_transform_validation_codes_are_preserved() {
    let compiler = Compiler::new();

    let cnn = "Within ml: Let model be Build cnn for 0 classes input channels 3. End.";
    assert_eq!(compiler.hir(cnn).unwrap_err().code, "S337");

    let freeze = format!(
        "{}Let frozen be Freeze model component \"\". End.",
        ml_prefix()
    );
    assert_eq!(compiler.hir(&freeze).unwrap_err().code, "S337");

    let lora = format!(
        "{}Let adapted be Apply lora to model rank 0 alpha 16. End.",
        ml_prefix()
    );
    assert_eq!(compiler.hir(&lora).unwrap_err().code, "S337");

    let qlora = format!(
        "{}Let adapted be Adapt model with qlora rank 8 alpha 16 quantization 3 bits targets \"q_proj\". End.",
        ml_prefix()
    );
    assert_eq!(compiler.hir(&qlora).unwrap_err().code, "S339");
}

#[test]
fn ml_resource_training_plan_and_metric_validation_are_preserved() {
    let compiler = Compiler::new();

    let budget = "Within ml: Let resources be Budget resources memory 0 gb workers 2. End.";
    assert_eq!(compiler.hir(budget).unwrap_err().code, "S342");

    let workers = "Within ml: Let resources be Budget resources memory 8 gb workers 1.5. End.";
    assert_eq!(compiler.hir(workers).unwrap_err().code, "S342");

    let precision = "Within ml: Let config be Plan training for 2 epochs using optimizer \"adam\" learning rate 0.001 batch size 8 seed 1 precision \"fp8\" accumulate 1 steps checkpoint every 10 steps. End.";
    assert_eq!(compiler.hir(precision).unwrap_err().code, "S338");

    let accumulation = "Within ml: Let config be Plan training for 2 epochs using optimizer \"adam\" learning rate 0.001 batch size 8 seed 1 precision \"fp16\" accumulate 0 steps checkpoint every 10 steps. End.";
    assert_eq!(compiler.hir(accumulation).unwrap_err().code, "S338");

    let metric = format!(
        "{}Let evaluation be Evaluate model on dataset using device metric \"bleu\". End.",
        ml_prefix()
    );
    assert_eq!(compiler.hir(&metric).unwrap_err().code, "S341");
}

#[test]
fn opaque_training_config_cannot_be_forged_from_string() {
    let compiler = Compiler::new();
    let source = format!(
        "{}Let run be Fit model using dataset on device with \"fake-config\". End.",
        ml_prefix()
    );
    assert_eq!(compiler.hir(&source).unwrap_err().code, "S329");
}

#[test]
fn advanced_ml_plan_preserves_configuration_and_lineage() {
    let compiler = Compiler::new();
    let source = format!(
        "{}Let config be Plan training for 3 epochs using optimizer \"adamw\" learning rate 0.0003 batch size 16 seed 42 precision \"bf16\" accumulate 4 steps checkpoint every 100 steps. Let run be Fit model using dataset on device with config. End.",
        ml_prefix()
    );
    let json = compiler.runtime_plan(&source).unwrap().to_json().to_string();

    assert!(json.contains("plan_training"));
    assert!(json.contains("gradient_accumulation"));
    assert!(json.contains("checkpoint_every"));
    assert!(json.contains("bf16"));
    assert!(json.contains("\\\"domain\\\":\\\"ml\\\""));
    assert!(json.contains("\\\"name\\\":\\\"training_run\\\""));
}
