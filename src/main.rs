mod model;

use itertools::Itertools;
use model::QuestionnaireResponse;
use parquet::{
    file::{reader::FileReader, serialized_reader::SerializedFileReader},
    record::RowAccessor,
    schema::types::Type,
};
use redis::Commands;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Value};
use std::{collections::HashMap, path::Path};

const PATIENT_ID: &str = "0f06d23e-bb59-5ef5-8089-d3f5eed44146";
const SYSTEM: &str = "tuftscentricity";
const RESOURCE: &str = "questionnaireresponse";

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Number {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum JsonType<'a> {
    Null,
    Number(Number),
    String(&'a str),
    Object(HashMap<&'a str, Box<JsonType<'a>>>),
    Array(Vec<Box<JsonType<'a>>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterValue<T> {
    Equal(T),
    NotEqual(T),
    Like(T),
    NotLike(T),
    In(Vec<T>),
    NotIn(Vec<T>),
    Or(T),
    Null,
    NotNull,
}

#[derive(Clone, Debug)]
pub struct DataFilter<T> {
    pub field: String,
    pub value: FilterValue<T>,
}

fn get_conn() -> redis::Connection {
    let client = redis::Client::open("redis://fhirdbsvr.wiise.azure.net:6379").unwrap();
    client.get_connection().unwrap()
}

async fn pull(key: &str) -> Vec<String> {
    let mut conn = get_conn();

    let len = conn.llen::<&str, isize>(key).unwrap();
    conn.lrange::<&str, Vec<String>>(key, 0, len).unwrap()
}

async fn apply_partition_filter(files: Vec<String>, filter: &DataFilter<String>) -> Vec<String> {
    let filtered_files = files
        .into_iter()
        .filter(|file| {
            let (partition, _file_name) =
                file.splitn(2, '/').collect_tuple::<(&str, &str)>().unwrap();
            let (partition_key, partition_value) = partition
                .splitn(2, '=')
                .collect_tuple::<(&str, &str)>()
                .unwrap();

            match partition_key.eq(&filter.field) {
                true => match &filter.value {
                    FilterValue::Equal(val) => val.eq(&partition_value),
                    FilterValue::NotEqual(val) => !val.eq(&partition_value),
                    FilterValue::In(val) => val.contains(&partition_value.to_string()),
                    FilterValue::NotIn(val) => !val.contains(&partition_value.to_string()),
                    FilterValue::Like(val) => partition_value.contains(val),
                    FilterValue::NotLike(val) => !partition_value.contains(val),
                    _ => false,
                },
                false => false,
            }
        })
        .collect_vec();
    filtered_files
}

async fn prepare_files(
    table_path: &str,
    db_name: &str,
    resource_name: &str,
    filters: &[DataFilter<String>],
) -> Vec<String> {
    let partition_columns =
        pull(format!("{}/{}/partition_columns", db_name, resource_name).as_str()).await;

    println!("Key = {}/{}/partition_columns", db_name, resource_name);
    println!("Partition columns: {:?}", partition_columns);
    let all_files = pull(format!("{}/{}/files", db_name, resource_name).as_str()).await;
    println!("All files: {:?}", all_files);

    let partition_column = partition_columns.first().unwrap();

    let partition_filter = filters
        .iter()
        .find(|filter| filter.field.eq(partition_column))
        .unwrap();

    let raw_files = apply_partition_filter(all_files, partition_filter).await;

    let files = raw_files
        .into_iter()
        .map(|file| {
            Path::new(&table_path)
                .join(file)
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect_vec();

    files
}

async fn get_file(file_path: String, filters: &[DataFilter<String>]) -> Vec<Value> {
    println!("Reading: {}", file_path);
    let file = match std::fs::File::open(file_path.as_str()) {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to read: {}", file_path);
            println!("Error: {}", e);
            return vec![];
        }
    };

    let reader = SerializedFileReader::new(file).unwrap();
    let metadata = reader.metadata();
    let mut selected_fields = metadata.file_metadata().schema().get_fields().to_vec();

    let schema_projection = Type::group_type_builder("spark_schema")
        .with_fields(&mut selected_fields)
        .build()
        .unwrap();

    let row_groups = vec![];
    let mut vector = vec![];

    // Iterate through row groups. Read the data and apply filters
    for num_row_group in 0..metadata.num_row_groups() {
        if !row_groups.is_empty() && !row_groups.contains(&(num_row_group as u8)) {
            continue;
        }
        let row_group = reader.get_row_group(num_row_group).unwrap();

        for row in row_group
            .get_row_iter(Some(schema_projection.clone()))
            .unwrap()
        {
            let mut skip_row = false;
            for filter in filters {
                let index_option = schema_projection
                    .get_fields()
                    .iter()
                    .find_position(|field| filter.field.eq(field.name()));

                let index = match index_option {
                    Some(tuple) => tuple.0,
                    None => {
                        continue;
                    }
                };
                let is_match = match &filter.value {
                    FilterValue::Equal(val) => row.get_string(index).unwrap().eq(val),
                    FilterValue::NotEqual(val) => !row.get_string(index).unwrap().eq(val),
                    FilterValue::Or(val) => row.get_string(index).unwrap().eq(val),
                    FilterValue::In(val) => val.contains(row.get_string(index).unwrap()),
                    FilterValue::NotIn(val) => !val.contains(row.get_string(index).unwrap()),
                    FilterValue::Like(val) => row.get_string(index).unwrap().contains(val),
                    FilterValue::NotLike(val) => !row.get_string(index).unwrap().contains(val),
                    FilterValue::Null => row.get_string(index).is_err(),
                    FilterValue::NotNull => row.get_string(index).is_ok(),
                };

                if !is_match {
                    skip_row = true;
                    break;
                }
            }
            if !skip_row {
                vector.push(row.to_json_value());
            }
        }
    }

    vector
}

async fn fetch_data() -> Vec<Value> {
    let base_path = "/mnt/wiise-etl/datalake/integrationarchive";
    let db_name = format!("integrationarchive{}", SYSTEM);

    let table_path = Path::new(base_path)
        .join(db_name.as_str())
        .join(RESOURCE)
        .to_str()
        .unwrap()
        .to_string();

    let patient_id = PATIENT_ID.to_string();

    let filters = vec![DataFilter {
        field: "yy__patient_id".to_string(),
        value: FilterValue::Equal(patient_id.clone()),
    }];

    let files = prepare_files(table_path.as_str(), db_name.as_str(), RESOURCE, &filters).await;

    println!("reading files started");
    let futures = files
        .into_iter()
        .map(|file| get_file(file.clone(), &filters));

    let results = futures::future::join_all(futures).await;
    println!("reading files finished");

    results.into_iter().flatten().collect_vec()
}

#[actix::main]
async fn main() {
    let parsed = fetch_data()
        .await
        .into_iter()
        .filter_map(
            |row| match from_value::<QuestionnaireResponse>(row.clone()) {
                Ok(q) => Some(q),
                Err(e) => {
                    println!("Error: {:?}", e);
                    println!("Value: {:?}", row);
                    None
                }
            },
        )
        .collect_vec();

    println!("ALL DONE\n{:?}", parsed);
}
