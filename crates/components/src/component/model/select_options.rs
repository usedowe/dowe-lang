#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOptionEach {
    pub item: String,
    pub collection: String,
    pub key: String,
    pub value: String,
    pub label: String,
    pub description: Option<String>,
}

