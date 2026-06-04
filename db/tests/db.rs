use db::DB;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

#[tokio::test]
async fn put_get_mulit_get_prefix_all() {
    let key = "key".to_string();
    let value = "value".to_string();
    let cf = "family";

    let dir = tempdir().unwrap();
    let db = DB::open(dir.path(), vec![cf]).await.unwrap();

    db.put(key.clone(), &value, cf).await.unwrap();

    let test: Option<String> = db.get(key.clone(), cf).await.unwrap();
    assert_eq!(test.unwrap(), value);

    let result: Vec<String> = db.multi_get(vec![key], cf).await.unwrap();
    assert_eq!(result, vec![value.clone()]);

    let result: Vec<String> = db.prefix_all("k".to_string(), cf).await.unwrap();
    assert_eq!(result, vec![value.clone()]);

    let result: Vec<String> = db.all(cf).await.unwrap();
    assert_eq!(result, vec![value]);
}

#[tokio::test]
#[should_panic]
async fn map_error() {
    let dir = tempdir().unwrap();
    let db = DB::open(dir.path(), vec![]).await.unwrap();
    let _: Option<String> = db.get("key".to_string(), "not_existed").await.unwrap();
}

#[tokio::test]
#[should_panic]
async fn invalid_value() {
    #[derive(Serialize, Deserialize, Debug)]
    struct Test {
        value: u64,
    }

    let dir = tempdir().unwrap();
    let db = DB::open(dir.path(), vec!["test"]).await.unwrap();
    db.put("key".to_string(), &Test { value: 10 }, "test")
        .await
        .unwrap();
    let _: Option<String> = db.get("key".to_string(), "test").await.unwrap();
}

