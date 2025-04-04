use crate::exports::edgee::components::data_collection::Dict;
use anyhow::Context;
use aws_credential_types::Credentials;
use aws_sigv4::http_request::{
    sign, PayloadChecksumKind, SignableBody, SignableRequest, SigningParams, SigningSettings,
};
use aws_sigv4::sign::v4;
use aws_smithy_runtime_api::client::identity::Identity;
use chrono::offset::Utc;
use chrono::DateTime;
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

pub struct Settings {
    pub access_key: String,
    pub secret_key: String,
    pub session_token: String, // could be empty
    pub region: String,
    pub bucket: String,
    pub key_prefix: String, // could be empty
}

impl Settings {
    pub fn new(settings_dict: Dict) -> anyhow::Result<Self> {
        let settings_map: HashMap<String, String> = settings_dict
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        let access_key = settings_map
            .get("aws_access_key")
            .context("Missing AWS Access Key")?
            .to_string();

        let secret_key = settings_map
            .get("aws_secret_key")
            .context("Missing AWS Secret Key")?
            .to_string();

        let session_token = settings_map
            .get("aws_session_token")
            .map(String::to_string)
            .unwrap_or_default(); // optional

        let region = settings_map
            .get("aws_region")
            .context("Missing AWS region")?
            .to_string();

        let bucket = settings_map
            .get("s3_bucket")
            .context("Missing S3 bucket")?
            .to_string();

        let key_prefix = settings_map
            .get("s3_key_prefix")
            .map(String::to_string)
            .unwrap_or_default(); // optional

        Ok(Self {
            access_key,
            secret_key,
            session_token,
            region,
            bucket,
            key_prefix,
        })
    }

    pub fn generate_random_s3_key() -> String {
        let datetime: DateTime<Utc> = SystemTime::now().into();
        format!(
            "{}-{}.json",
            datetime.format("%Y-%m-%d-%H-%M-%S"),
            Uuid::new_v4(),
        )
    }

    pub fn generate_s3_url(&self) -> String {
        format!(
            "https://{}/{}{}",
            self.generate_s3_host(),
            self.key_prefix.clone(), // could be empty
            Self::generate_random_s3_key(),
        )
    }

    pub fn generate_s3_host(&self) -> String {
        format!(
            "{}.s3.{}.amazonaws.com",
            self.bucket.clone(),
            self.region.clone(),
        )
    }

    pub fn generate_s3_headers(
        &self,
        s3_url: String,
        file_content: String,
    ) -> Vec<(String, String)> {
        let session_token = if self.session_token.is_empty() {
            None
        } else {
            Some(self.session_token.clone())
        };

        // create Identity with static Credentials
        let identity: Identity = Credentials::from_keys(
            self.access_key.clone(),
            self.secret_key.clone(),
            session_token,
        )
        .into();

        let mut signing_settings = SigningSettings::default();
        // enable required header for s3
        signing_settings.payload_checksum_kind = PayloadChecksumKind::XAmzSha256;

        // build signing parameters
        let signing_params: SigningParams = v4::SigningParams::builder()
            .identity(&identity)
            .region(self.region.as_str())
            .name("s3")
            .time(SystemTime::now())
            .settings(signing_settings)
            .build()
            .unwrap()
            .into();

        // create a signable request
        let signable_request = SignableRequest::new(
            "PUT",
            s3_url,
            std::iter::empty(),
            SignableBody::Bytes(file_content.as_bytes()),
        )
        .expect("signable request");

        // generate the signature headers
        let (signing_instructions, _signature) = sign(signable_request, &signing_params)
            .unwrap()
            .into_parts();

        // convert to Vec<(String, String)>
        let mut headers: Vec<(String, String)> = signing_instructions
            .headers()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        headers.extend(vec![("host".to_string(), self.generate_s3_host())]);

        headers
    }
}
