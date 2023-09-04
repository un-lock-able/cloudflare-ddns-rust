mod request {
    use serde::Serialize;

    use crate::domain_record_changer::RecordType;

    #[derive(Serialize)]
    struct DescribeRecord {
        name: String,
        #[serde(rename = "type")]
        record_type: RecordType,
    }
}

pub mod response {
    use serde::Deserialize;
    use crate::domain_record_changer::RecordType;
    
    #[derive(Deserialize, Debug)]
    pub struct RecordDetail {
        pub content: String,
        pub name: String,
        pub proxied: bool,
        #[serde(rename = "type")]
        pub record_type: RecordType,
        pub id: String,
        pub locked: bool,
        pub proxiable: bool,
        pub ttl: Option<u32>,
        pub zone_name: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct CodeMessagePair {
        code: u32,
        message: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct ResultInfo {
        pub count: u32,
        pub page: u32,
        pub per_page: u32,
        pub total_count: u32,
    }
    
    #[derive(Deserialize, Debug)]
    pub struct DescribeRecord {
        pub result: Vec<RecordDetail>,
        pub errors: Vec<CodeMessagePair>,
        pub messages: Vec<CodeMessagePair>,
        pub success: bool,
        pub result_info: ResultInfo,
    }

    impl std::fmt::Display for CodeMessagePair{
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Code {}: {}", self.code, self.message)
        }
    }
}
