use std::str::FromStr;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::serde::{Deserialize, Serialize};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, near_sdk::PanicOnDefault)]
pub struct CertificatesContract {
    certificate_issuers: near_sdk::collections::LookupMap<near_sdk::AccountId, IssuerProfile>,
    certificate_templates: near_sdk::collections::Vector<CertificateTemplate>,
    certificates: near_sdk::collections::Vector<Certificate>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct IssuerProfile {
    display_name: String,
}

type CertificateTemplateId = near_sdk::json_types::U64;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CertificateTemplate {
    template_kind: String,
    template: near_sdk::json_types::Base64VecU8,
}

type CertificateId = near_sdk::json_types::U64;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(
    crate = "near_sdk::serde",
    tag = "kind",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
pub enum CertificateStatus {
    New,
    Revoked { reason: String },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Certificate {
    issuer_account_id: near_sdk::AccountId,
    status: CertificateStatus,
    certificate_template_id: CertificateTemplateId,
    /// Cerfificate Data is a binary blob that is usually a JSON object encrypted with NaCl.
    ///
    /// ```
    /// #[derive(Deserialize, Serialize)]
    /// struct CertificateData {
    ///     /// Team name or a person full name
    ///     issued_for_display_name: String,
    ///     /// Kind of the certificate template, e.g. "svg-template"
    ///     certificate_template_kind: String,
    ///     /// Template or URL to template that should be used to render visual certificate, e.g. SVG file with
    ///     /// placeholders
    ///     certificate_template: String,
    ///     /// Key-value pairs that should be passed to certificate template
    ///     certificate_fields: std::collections::HashMap<String, String>,
    /// }
    /// ```
    encrypted_certificate_data: Vec<u8>,
    /// This field is used to store the nonce that was used to encrypt the certificate data.
    certificate_encryption_nonce: Vec<u8>,
    /// Issuer might want to encrypt and put the key that will be able to read the encrypted
    /// certificate data here.
    ///
    /// WARNING: Do you put unencrypted key here since that will defeat the purpose of having
    /// certificate encryption in the first place.
    certificate_encryption_recovery: Option<Vec<u8>>,
}

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Web4Request {
    //#[serde(rename = "accountId")]
    //pub account_id: Option<String>,
    pub path: String,
    //pub params: std::collections::HashMap<String, String>,
    //pub query: std::collections::HashMap<String, Vec<String>>,
    pub preloads: std::collections::HashMap<String, Web4Response>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde", untagged)]
pub enum Web4Response {
    Body {
        #[serde(rename = "contentType")]
        content_type: String,
        body: near_sdk::json_types::Base64VecU8,
    },
    BodyUrl {
        #[serde(rename = "bodyUrl")]
        body_url: String,
    },
    PreloadUrls {
        #[serde(rename = "preloadUrls")]
        preload_urls: Vec<String>,
    },
}

#[near_bindgen]
impl CertificatesContract {
    #[init]
    pub fn new() -> Self {
        Self {
            certificate_issuers: near_sdk::collections::LookupMap::new(b"i"),
            certificate_templates: near_sdk::collections::Vector::new(b"t"),
            certificates: near_sdk::collections::Vector::new(b"c"),
        }
    }

    #[private]
    pub fn register_issuer(
        &mut self,
        issuer_account_id: near_sdk::AccountId,
        issuer_profile: IssuerProfile,
    ) {
        self.certificate_issuers
            .insert(&issuer_account_id, &issuer_profile);
    }

    pub fn register_certificate_template(
        &mut self,
        certificate_template: CertificateTemplate,
    ) -> CertificateTemplateId {
        let issuer_account_id = near_sdk::env::predecessor_account_id();
        assert!(
            self.certificate_issuers.contains_key(&issuer_account_id),
            "only registered issuers can add new certificate templates"
        );

        let certificate_template_id = self.certificate_templates.len();
        self.certificate_templates.push(&certificate_template);

        certificate_template_id.into()
    }

    pub fn issue_certificates(&mut self, certificates: Vec<Certificate>) -> Vec<CertificateId> {
        let issuer_account_id = near_sdk::env::predecessor_account_id();
        assert!(
            self.certificate_issuers.contains_key(&issuer_account_id),
            "only registered issuers can submit certificates"
        );
        certificates
            .into_iter()
            .map(|certificate| {
                assert_eq!(
                    certificate.issuer_account_id, issuer_account_id,
                    "the issuer account id on the certificate does not match the issuer account id"
                );
                let certificate_id = self.certificates.len();
                self.certificates.push(&certificate);
                certificate_id.into()
            })
            .collect()
    }

    pub fn revoke_certificates(
        &mut self,
        revoking_certificate_ids: Vec<CertificateId>,
        reason: String,
    ) {
        let issuer_account_id = near_sdk::env::predecessor_account_id();
        let is_contract_owner = issuer_account_id == near_sdk::env::current_account_id();
        assert!(
            is_contract_owner || self.certificate_issuers.contains_key(&issuer_account_id),
            "only contract owner and registered issuers can revoke certificates"
        );
        for revoking_certificate_id in revoking_certificate_ids {
            let mut certificate = self
                .certificates
                .get(revoking_certificate_id.into())
                .expect("certificate must exist to be revoked");
            if !is_contract_owner {
                assert_eq!(
                    certificate.issuer_account_id, issuer_account_id,
                    "only contract owner and the certificate issuer can revoke certificates"
                );
            }
            certificate.status = CertificateStatus::Revoked {
                reason: reason.clone(),
            };
            self.certificates
                .replace(revoking_certificate_id.into(), &certificate);
        }
    }

    pub fn get_certificate(&self, certificate_id: CertificateId) -> Option<Certificate> {
        self.certificates.get(certificate_id.into())
    }

    /// Learn more about web4 here: https://web4.near.page
    pub fn web4_get(&self, request: Web4Request) -> Web4Response {
        if request.path == "/" {
            Web4Response::Body {
                content_type: "text/html; charset=UTF-8".to_owned(),
                body: "<h1>Welcome to Certification on NEAR</h1><p>Reach out to <a href=\"https://t.me/frolvlad\">https://t.me/frolvlad</a> for more details about this project</p>".as_bytes().to_owned().into(),
            }
        } else {
            if let Ok(certificate_id) = u64::from_str(&request.path[1..]) {
                if let Some(certificate) = self.get_certificate(certificate_id.into()) {
                let certificate_template = self.certificate_templates.get(certificate.certificate_template_id.into()).unwrap();
                    Web4Response::Body {
                        content_type: "text/html; charset=UTF-8".to_owned(),
                        body: include_str!("certificate-viewer.html")
                            .replace(
                                "{{{ CERTIFICATE_ID }}}",
                                &certificate_id.to_string(),
                            )
                            .replace(
                                "{{{ CERTIFICATE_TEMPLATE }}}",
                                &serde_json::to_string(&certificate_template).unwrap()
                            )
                            .replace(
                                "{{{ CERTIFICATE }}}",
                                &serde_json::to_string(&certificate).unwrap(),
                            )
                            .to_owned()
                            .into_bytes()
                            .into(),
                    }
                } else {
                    Web4Response::Body {
                        content_type: "text/html; charset=UTF-8".to_owned(),
                        body: "<h1>Certificate Not Found</h1><p>Go to the <a href=\"/\">home page</a></p>".as_bytes().to_owned().into(),
                    }
                }
            } else {
                Web4Response::Body {
                    content_type: "text/html; charset=UTF-8".to_owned(),
                    body: "<h1>Invalid certificate ID</h1><p>Go to the <a href=\"/\">home page</a></p>".as_bytes().to_owned().into(),
                }
            }
        }
    }
}
