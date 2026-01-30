use crate::account::test_fixtures::account_fixture;

// #[tokio::test]
// async fn store_account() {
//     let account = account_fixture();
//
//     let tempdir = tempfile::tempdir().unwrap();
//     let path = tempdir.path().join("test.txt");
//
//     let account_storage = OnDiskAccountStorage::new(path);
//     account_storage
//         .store_account(account.clone())
//         .await
//         .unwrap();
//
//     let stored_accounts = account_storage.load_accounts().await.unwrap();
//     assert_eq!(vec![account], stored_accounts);
// }
//
// #[tokio::test]
// async fn store_twice_overwrites_for_same_mode() {
//     let account = account_fixture();
//
//     let tempdir = tempfile::tempdir().unwrap();
//     let path = tempdir.path().join("test.txt");
//     let account_storage = OnDiskAccountStorage::new(path);
//
//     account_storage
//         .store_account(account.clone())
//         .await
//         .unwrap();
//     account_storage
//         .store_account(account.clone())
//         .await
//         .unwrap();
//
//     let stored_accounts = account_storage.load_accounts().await.unwrap();
//     assert_eq!(vec![account], stored_accounts);
// }
//
// #[tokio::test]
// async fn load_returns_empty_if_file_does_not_exist() {
//     let tempdir = tempfile::tempdir().unwrap();
//     let path = tempdir.path().join("test.txt");
//     let account_storage = OnDiskAccountStorage::new(path);
//
//     let result = account_storage.load_accounts().await;
//     assert!(matches!(result, Ok(v) if v.is_empty()));
// }
//
// #[tokio::test]
// async fn load_fails_if_file_contains_invalid_json() {
//     let tempdir = tempfile::tempdir().unwrap();
//     let path = tempdir.path().join("test.txt");
//     let account_storage = OnDiskAccountStorage::new(path.clone());
//
//     // Write invalid JSON so serde_json definitely errors
//     std::fs::write(&path, b"not json").unwrap();
//
//     let result = account_storage.load_accounts().await;
//     assert!(matches!(
//             result,
//             Err(OnDiskMnemonicStorageError::ReadError(_))
//         ));
// }
//
// #[tokio::test]
// async fn load_of_legacy_single_account_json_still_works() -> anyhow::Result<()> {
//     let account = account_fixture();
//
//     // Legacy shape supported: a single `StoredAccount` JSON object (not the map).
//     let legacy_single = StoredAccount {
//         name: "foomp".to_string(),
//         mnemonic: account.mnemonic.clone(),
//         mode: account.mode,
//         nonce: 0,
//     };
//
//     let tempdir = tempfile::tempdir()?;
//     let path = tempdir.path().join("test.txt");
//
//     let file = OpenOptions::new()
//         .create_new(true)
//         .write(true)
//         .open(&path)?;
//     serde_json::to_writer(file, &legacy_single)?;
//
//     let account_storage = OnDiskAccountStorage::new(path);
//     let loaded = account_storage.load_accounts().await?;
//     assert_eq!(vec![account], loaded);
//
//     Ok(())
// }
