pub mod request {
    use crate::RecordType;
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct CreateRecord {
        pub name: String,
        pub content: String,
        #[serde(rename = "type")]
        pub record_type: RecordType,
        pub proxied: bool,
        pub ttl: u32,
    }

    #[derive(Serialize)]
    pub struct UpdateRecord {
        pub name: String,
        pub content: String,
        #[serde(rename = "type")]
        pub record_type: RecordType,
        pub proxied: bool,
        pub ttl: u32,
    }
}

pub mod response {
    use crate::RecordType;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct CodeMessagePair {
        pub code: u32,
        pub message: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct MetaInfo {
        pub auto_added: Option<bool>,
        pub source: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    pub struct ResultInfo {
        pub count: u32,
        pub page: u32,
        pub per_page: u32,
        pub total_count: u32,
    }

    #[derive(Deserialize, Debug)]
    pub struct RecordDetail {
        pub content: String,
        pub name: String,
        pub proxied: Option<bool>,
        #[serde(rename = "type")]
        pub record_type: RecordType,
        pub comment: Option<String>,
        pub created_on: String,
        pub id: String,
        pub meta: Option<MetaInfo>,
        pub modified_on: String,
        pub proxiable: bool,
        pub tags: Vec<String>,
        pub ttl: u32,
        pub zone_name: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct DescribeRecord {
        pub result: Vec<RecordDetail>,
        pub errors: Vec<CodeMessagePair>,
        pub messages: Vec<CodeMessagePair>,
        pub success: bool,
        pub result_info: ResultInfo,
    }

    #[derive(Deserialize, Debug)]
    pub struct CreateRecord {
        pub result: RecordDetail,
        pub errors: Vec<CodeMessagePair>,
        pub messages: Vec<CodeMessagePair>,
        pub success: bool,
    }

    #[derive(Deserialize, Debug)]
    pub struct UpdateRecord {
        pub result: RecordDetail,
        pub errors: Vec<CodeMessagePair>,
        pub messages: Vec<CodeMessagePair>,
        pub success: bool,
    }

    impl std::fmt::Display for CodeMessagePair {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Code {}: {}", self.code, self.message)
        }
    }
}
