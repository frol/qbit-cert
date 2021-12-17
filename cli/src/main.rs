use std::io::Read;

use clap::Parser;
use near_jsonrpc_client::{methods, JsonRpcClient};
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::secretbox;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(long)]
    certification_center_account_id: near_primitives::types::AccountId,

    #[clap(long)]
    certification_center_secret_key: near_crypto::SecretKey,

    #[clap(long)]
    certification_contract_account_id: near_primitives::types::AccountId,

    /// Name of the person or the team
    #[clap(long)]
    issued_for_display_name: String,

    #[clap(long)]
    template_kind: String,

    #[clap(long)]
    template_path: std::path::PathBuf,

    #[clap(long)]
    template_fields: serde_json::Value,
}

#[derive(Deserialize, Serialize)]
struct CertificateData {
    /// Team name or a person full name
    issued_for_display_name: String,
    /// Kind of the certificate template, e.g. "svg-template"
    certificate_template_kind: String,
    /// Template or template URL that should be used to render visual certificate, e.g. SVG file with
    /// placeholders
    certificate_template: String,
    /// Key-value pairs that should be passed to certificate template
    certificate_fields: std::collections::HashMap<String, String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    //let near_jsonrpc_client = JsonRpcClient::connect("https://rpc.testnet.near.org");
    let near_jsonrpc_client = JsonRpcClient::connect("https://rpc.mainnet.near.org");

    let certification_center_access_key_request = methods::query::RpcQueryRequest {
        block_reference: near_primitives::types::Finality::Final.into(),
        request: near_primitives::views::QueryRequest::ViewAccessKey {
            account_id: args.certification_center_account_id.clone(),
            public_key: args.certification_center_secret_key.public_key(),
        },
    };
    let certification_center_access_key_response = near_jsonrpc_client
        .call(certification_center_access_key_request)
        .await?;
    let recent_block_hash = certification_center_access_key_response.block_hash;
    let certification_center_access_key =
        if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(access_key) =
            certification_center_access_key_response.kind
        {
            access_key
        } else {
            anyhow::bail!(
                "unexpected response when queried for certification center access key: {:#?}",
                certification_center_access_key_response.kind
            );
        };

    let certificate_center_signer = near_crypto::InMemorySigner::from_secret_key(
        args.certification_center_account_id.clone(),
        args.certification_center_secret_key.clone(),
    );

    /*
    let mut certificate_template = Vec::new();
    let mut certificate_template_file = std::fs::File::open(args.template_path)?;
    certificate_template_file.read_to_end(&mut certificate_template)?;
    */
    let certificate_template = std::fs::read_to_string(args.template_path)?;

    let certificate_fields = serde_json::from_value(args.template_fields)?;

    let certificate_data = CertificateData {
        issued_for_display_name: args.issued_for_display_name,
        certificate_template_kind: args.template_kind,
        certificate_template,
        certificate_fields,
    };

    let certificate_secret_key = secretbox::gen_key();
    let certificate_encryption_nonce = secretbox::gen_nonce();
    let encrypted_certificate_data = secretbox::seal(
        serde_json::to_string(&certificate_data)?.as_bytes(),
        &certificate_encryption_nonce,
        &certificate_secret_key,
    );
    //let their_plaintext = secretbox::open(&ciphertext, &nonce, &key).unwrap();
    //assert!(plaintext == &their_plaintext[..]);
    println!(
        "Len: {}\n{}\n{}\n{}",
        encrypted_certificate_data.len(),
        hex::encode(&encrypted_certificate_data),
        hex::encode(certificate_encryption_nonce.0),
        hex::encode(certificate_secret_key.as_ref())
    );

    let signed_transaction = near_primitives::transaction::Transaction::new(
        args.certification_center_account_id.clone(),
        args.certification_center_secret_key.public_key(),
        args.certification_contract_account_id,
        certification_center_access_key.nonce + 1,
        recent_block_hash,
    )
    .function_call(
        "issue_certificates".to_owned(),
        serde_json::to_string(&serde_json::json!({
            "certificates": [
                {
                    "issuer_account_id": args.certification_center_account_id,
                    "status": {
                        "kind": "NEW",
                    },
                    "encrypted_certificate_data": encrypted_certificate_data,
                    "certificate_encryption_nonce": certificate_encryption_nonce.0,
                    //"encrypted_certificate_public_key": encrypted_certificate_public_key,
                },
            ],
        }))?
        .into_bytes(),
        near_primitives::types::Gas::from(150_000_000_000_000u64),
        near_primitives::types::Balance::from(0u128),
    )
    .sign(&certificate_center_signer);

    let broadcast_tx_commit_request =
        methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest { signed_transaction };
    let tx_status = near_jsonrpc_client
        .call(broadcast_tx_commit_request)
        .await?;

    println!("{:?}", tx_status);

    Ok(())
}
