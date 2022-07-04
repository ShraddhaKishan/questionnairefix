use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: Option<String>,
    pub extension: Option<Vec<Value>>,
    pub content_type: Option<String>,
    pub language: Option<String>,
    pub data: Option<String>,
    pub url: Option<String>,
    pub size: Option<u64>,
    pub hash: Option<String>,
    pub title: Option<String>,
    pub creation: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Coding {
    pub id: Option<String>,
    pub extension: Option<Vec<Value>>,
    pub system: Option<String>,
    pub version: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
    #[serde(rename = "userSelected")]
    pub user_selected: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Period {
    pub id: Option<String>,
    pub extension: Option<Vec<Value>>,
    pub start: Option<String>,
    pub end: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CodeableConcept {
    pub id: Option<String>,
    pub extension: Option<Vec<Value>>,
    pub coding: Option<Vec<Coding>>,
    pub text: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Identifier {
    pub id: Option<String>,
    pub extension: Option<Vec<Value>>,
    pub r#use: Option<String>,
    pub r#type: Option<CodeableConcept>,
    pub system: Option<String>,
    pub value: Option<String>,
    pub period: Option<Period>,
    pub assigner: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Reference {
    pub id: Option<String>,
    pub extension: Option<Vec<Value>>,
    pub reference: Option<String>,
    pub r#type: Option<String>,
    pub identifier: Option<Identifier>,
    pub display: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Quantity {
    pub id: Option<String>,
    pub extension: Option<Vec<String>>,
    pub value: Option<f32>,
    pub comparator: Option<String>,
    pub unit: Option<String>,
    pub system: Option<String>,
    pub code: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Questionnaire {
    pub id: Option<String>,
    pub url: Option<String>,
    pub name: Option<String>,
    pub title: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnswerBackboneElement {
    pub value_boolean: Option<bool>,
    pub value_decimal: Option<f64>,
    pub value_integer: Option<i32>,
    pub value_date: Option<String>,
    pub value_date_time: Option<String>,
    pub value_time: Option<String>,
    pub value_string: Option<String>,
    pub value_uri: Option<String>,
    pub value_attachment: Option<Attachment>,
    pub value_coding: Option<Coding>,
    pub value_quantity: Option<Quantity>,
    pub value_reference: Option<Reference>,
    pub item: Option<Box<ItemBackboneElement>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemBackboneElement {
    pub link_id: String,
    pub definition: Option<String>,
    pub text: Option<String>,
    pub answer: Option<Vec<AnswerBackboneElement>>,
    pub item: Option<Box<Vec<Self>>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireResponse {
    pub id: Option<String>,
    pub contained: Option<Vec<Value>>,
    pub identifier: Option<Identifier>,
    pub based_on: Option<Vec<Reference>>,
    pub part_of: Option<Vec<Reference>>,
    pub questionnaire: Option<String>,
    pub status: String,
    pub subject: Option<Reference>,
    pub encounter: Option<Reference>,
    pub authored: Option<String>,
    pub author: Option<Reference>,
    pub source: Option<Reference>,
    pub item: Option<Vec<ItemBackboneElement>>,
}
