use serde_json::json;
use workspaces::prelude::*;

const STATUS_MSG_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/qbit_cert.wasm";

#[tokio::test]
async fn test_end_to_end() {
    let worker = workspaces::sandbox();
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH).unwrap();
    let contract = worker.dev_deploy(wasm).await.unwrap();

    let outcome = worker
        .call(
            &contract,
            "new".into(),
            json!({}).to_string().into_bytes(),
            None,
        )
        .await
        .unwrap();
    println!("new: {:?}", outcome);

    let result = worker
        .call(
            &contract,
            "register_issuer".into(),
            json!({
                "issuer_account_id": contract.id(),
                "issuer_profile": {
                    "display_name": "Qbit",
                },
            })
            .to_string()
            .into_bytes(),
            None,
        )
        .await
        .unwrap();
    println!("register_issuer: {:?}", &result);

    let result = worker
        .call(
            &contract,
            "issue_certificate".into(),
            json!({
                "certificate": {
                    "issuer_account_id": contract.id(),
                    "status": {
                        "kind": "NEW",
                    },
                    "encrypted_certificate_data": [],
                    "encrypted_certificate_public_key": vec![0u8; 32],
                },
            })
            .to_string()
            .into_bytes(),
            None,
        )
        .await
        .unwrap();
    println!("issue_certificate: {:?}", &result);

    let result = worker
        .view(
            contract.id().clone(),
            "get_certificate".into(),
            json!({ "certificate_id": 0usize }).to_string().into_bytes(),
        )
        .await
        .unwrap();
    println!(
        "get_certificate: {:?}",
        serde_json::to_string_pretty(&result).unwrap()
    );
}
