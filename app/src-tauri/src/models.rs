use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const DEFAULT_MODEL_ID: &str = "small.en-q5_1";
const HUGGING_FACE_BASE_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelStatus {
    NotDownloaded,
    Downloading,
    Downloaded,
    Selected,
    Loaded,
    Failed,
    UpdateAvailable,
}

impl ModelStatus {
    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::NotDownloaded => "not_downloaded",
            Self::Downloading => "downloading",
            Self::Downloaded => "downloaded",
            Self::Selected => "selected",
            Self::Loaded => "loaded",
            Self::Failed => "failed",
            Self::UpdateAvailable => "update_available",
        }
    }

    pub fn from_db_value(value: &str) -> Self {
        match value {
            "downloading" => Self::Downloading,
            "downloaded" => Self::Downloaded,
            "selected" => Self::Selected,
            "loaded" => Self::Loaded,
            "failed" => Self::Failed,
            "update_available" => Self::UpdateAvailable,
            _ => Self::NotDownloaded,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChecksumKind {
    Sha1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelChecksum {
    pub kind: ChecksumKind,
    pub value: String,
}

#[derive(Debug, Clone, Copy)]
pub struct CatalogModel {
    pub id: &'static str,
    pub name: &'static str,
    pub filename: &'static str,
    pub disk_size_label: &'static str,
    pub expected_sha1: Option<&'static str>,
}

impl CatalogModel {
    pub fn download_url(self) -> String {
        format!("{}/{}", HUGGING_FACE_BASE_URL, self.filename)
    }

    pub fn checksum(self) -> Option<ModelChecksum> {
        self.expected_sha1.map(|value| ModelChecksum {
            kind: ChecksumKind::Sha1,
            value: value.to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub download_url: String,
    pub disk_size_label: String,
    pub local_path: Option<String>,
    pub size_bytes: Option<u64>,
    pub status: ModelStatus,
    pub checksum: Option<ModelChecksum>,
    pub selected: bool,
    pub downloaded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ModelRecord {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub local_path: Option<String>,
    pub size_bytes: Option<u64>,
    pub status: ModelStatus,
    pub checksum: Option<String>,
    pub selected: bool,
    pub downloaded_at: Option<DateTime<Utc>>,
}

pub fn catalog() -> &'static [CatalogModel] {
    &[
        CatalogModel {
            id: "tiny.en",
            name: "Tiny English",
            filename: "ggml-tiny.en.bin",
            disk_size_label: "75 MiB",
            expected_sha1: Some("c78c86eb1a8faa21b369bcd33207cc90d64ae9df"),
        },
        CatalogModel {
            id: "base.en",
            name: "Base English",
            filename: "ggml-base.en.bin",
            disk_size_label: "142 MiB",
            expected_sha1: Some("137c40403d78fd54d454da0f9bd998f78703390c"),
        },
        CatalogModel {
            id: "small.en",
            name: "Small English",
            filename: "ggml-small.en.bin",
            disk_size_label: "466 MiB",
            expected_sha1: Some("db8a495a91d927739e50b3fc1cc4c6b8f6c2d022"),
        },
        CatalogModel {
            id: DEFAULT_MODEL_ID,
            name: "Small English Q5_1",
            filename: "ggml-small.en-q5_1.bin",
            disk_size_label: "181 MiB",
            expected_sha1: Some("20f54878d608f94e4a8ee3ae56016571d47cba34"),
        },
        CatalogModel {
            id: "medium.en",
            name: "Medium English",
            filename: "ggml-medium.en.bin",
            disk_size_label: "1.5 GiB",
            expected_sha1: Some("8c30f0e44ce9560643ebd10bbe50cd20eafd3723"),
        },
        CatalogModel {
            id: "large-v3-turbo-q5_0",
            name: "Large v3 Turbo Q5_0",
            filename: "ggml-large-v3-turbo-q5_0.bin",
            disk_size_label: "547 MiB",
            expected_sha1: Some("e050f7970618a659205450ad97eb95a18d69c9ee"),
        },
    ]
}

pub fn catalog_model(id: &str) -> Option<CatalogModel> {
    catalog().iter().copied().find(|model| model.id == id)
}
