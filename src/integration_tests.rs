#[cfg(test)]
mod tests {
    use crate::DBPool;
    use crate::auth::{authenticate, AuthData};
    use crate::routes::{
        lime_eth_transactions_hashes,
        lime_all,
        lime_eth_rlphex,
        lime_my
    };
    use crate::transaction::Transaction;
    use crate::setup;

    use actix_web::{test, App};
    use actix_web::web::Data;
    use actix_web::http::header::ContentType;
    use ctor::ctor;
    use diesel::prelude::*;
    use diesel::r2d2::{self, ConnectionManager};
    use serde_json::Value;
    use std::env;

    // Ran only once in order to setup the Test database
    #[ctor]
    fn run_db_migrations() {
        let pool = setup_test_db();
        let mut conn = pool.get().expect("Failed to get connection from pool.");
        setup::run_migrations(&mut conn);
    }

    fn setup_test_db() -> DBPool {
        let database_url = env::var("DB_CONNECTION_URL").expect("DB_CONNECTION_URL must be set");

        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.")
    }


    #[actix_web::test]
    async fn test_lime_eth_transaction_hashes() {
        let pool = setup_test_db();

        // Set up test server
        let mut app = test::init_service(App::new().app_data(Data::new(pool.clone())).service(lime_eth_transactions_hashes).service(lime_all)).await;

        // Gathered transaction from Sepolia-etherscan: https://sepolia.etherscan.io/tx/0x22f4a0e8243f6becd1e4e31fce147244a5de7ce604080854c516b324a186a59e
        // test purposes only - 0x22f4a0e8243f6becd1e4e31fce147244a5de7ce604080854c516b324a186a59e
        let example_transaction_hash = "0x22f4a0e8243f6becd1e4e31fce147244a5de7ce604080854c516b324a186a59e";

        // Retrieve transaction
        let uri = format!("/lime/eth?transactionHashes={}", example_transaction_hash);
        let req = test::TestRequest::get()
            .uri(&uri)
            .to_request();
        let resp: Value = test::call_and_read_body_json(&mut app, req).await;
        let transactions: Vec<Transaction> = serde_json::from_value(resp["transactions"].clone()).expect("Failed to parse transactions");
        assert!(!transactions.is_empty(), "No transactions found");

        let transaction = transactions.first().unwrap();
        assert_eq!(transaction.transaction_hash, example_transaction_hash);
    }

    #[actix_web::test]
    async fn test_lime_eth_rlp_hex() {
        let pool = setup_test_db();

        // Set up test server
        let mut app = test::init_service(App::new().app_data(Data::new(pool.clone())).service(lime_eth_rlphex)).await;

        // Gathered transaction from Sepolia-etherscan
        let first_transaction_hash = "0x6d61b62233334ebfb28515b4e2aa0e1011fdf542cf4948ef2831b65f0f1fe542";
        let _second_transaction_hash = "0x2f5bf15391119b4851c619f97d15ec9c0bd580eb034c29ce32e4c35bb3f288eb";
        // Hardcoded RLPHex encoded from https://toolkit.abdk.consulting/ethereum#rlp
        let rlp_hex = "0xf842a06d61b62233334ebfb28515b4e2aa0e1011fdf542cf4948ef2831b65f0f1fe542a02f5bf15391119b4851c619f97d15ec9c0bd580eb034c29ce32e4c35bb3f288eb";

        // Retrieve transaction
        let uri = format!("/lime/eth/{}", rlp_hex);
        let req = test::TestRequest::get()
            .uri(&uri)
            .to_request();
        let resp: Value = test::call_and_read_body_json(&mut app, req).await;

        let transactions: Vec<Transaction> = serde_json::from_value(resp["transactions"].clone()).expect("Failed to parse transactions");
        assert!(!transactions.is_empty(), "No transactions found");

        let transaction = transactions.first().unwrap();
        assert_eq!(transaction.transaction_hash, first_transaction_hash);
    }

    #[actix_web::test]
    async fn test_lime_all() {
        let pool = setup_test_db();

        // Set up test server
        let mut app = test::init_service(App::new().app_data(Data::new(pool.clone())).service(lime_eth_transactions_hashes).service(lime_all)).await;

        // Gathered transaction from Sepolia-etherscan: https://sepolia.etherscan.io/tx/0x22f4a0e8243f6becd1e4e31fce147244a5de7ce604080854c516b324a186a59e
        // test purposes only - 0x22f4a0e8243f6becd1e4e31fce147244a5de7ce604080854c516b324a186a59e
        let sample_tx_hash = "0x22f4a0e8243f6becd1e4e31fce147244a5de7ce604080854c516b324a186a59e";
        let sample_tx_hash_1 = "0xdb731feeb8b21013c77df06e3e2a2879db25c4258798b50b61f10867b61e7278";

        // Retrieve transaction
        let req = test::TestRequest::get()
            .uri(&format!("/lime/eth?transactionHashes={}&transactionHashes={}&transactionHashes={}",
                    sample_tx_hash, sample_tx_hash, sample_tx_hash_1))
            .to_request();
        test::call_service(&mut app, req).await;

        let req = test::TestRequest::get()
            .uri("/lime/all")
            .to_request();
        let resp: Value = test::call_and_read_body_json(&mut app, req).await;
        let transactions: Vec<Transaction> = serde_json::from_value(resp["transactions"].clone()).expect("Failed to parse transactions");

        assert!(transactions.len() >= 2, "Transactions stored are less than expected");
    }

    #[actix_web::test]
    async fn test_authenticate_success() {
        let mut app = test::init_service(App::new().service(authenticate)).await;
        let req = test::TestRequest::post()
            .uri("/lime/authenticate")
            .set_json(&AuthData {
                username: "alice".into(),
                password: "alice".into(),
            })
        .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_authenticate_failure() {
        let mut app = test::init_service(App::new().service(authenticate)).await;
        let req = test::TestRequest::post()
            .uri("/lime/authenticate")
            .set_json(&AuthData {
                username: "emo".into(),
                password: "emo".into(),
            })
        .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), 401);
    }

    #[actix_web::test]
    async fn test_lime_my() {
        let pool = setup_test_db();

        // Set up test server
        let mut app = test::init_service(
            App::new()
            .app_data(Data::new(pool.clone()))
            .service(lime_eth_rlphex)
            .service(lime_my)
            .service(authenticate))
            .await;

        // Get AUTH_TOKEN for `alice`
        let req = test::TestRequest::post()
            .uri("/lime/authenticate")
            .set_json(&AuthData {
                username: "alice".into(),
                password: "alice".into(),
            })
        .to_request();
        let resp: Value = test::call_and_read_body_json(&mut app, req).await;
        let auth_token = resp["token"].as_str().expect("Failed to get token");

        // Get some transaction searches for `alice`
        let tx_hash = "0x6d61b62233334ebfb28515b4e2aa0e1011fdf542cf4948ef2831b65f0f1fe542";
        let rlp_hex = "0xe1a06d61b62233334ebfb28515b4e2aa0e1011fdf542cf4948ef2831b65f0f1fe542";
        // Retrieve transaction
        let req = test::TestRequest::get()
            .uri(&format!("/lime/eth/{}", rlp_hex))
            .insert_header(("AUTH_TOKEN", auth_token))
            .insert_header(ContentType::json())
            .to_request();
        test::call_service(&mut app, req).await;

        // Get transaction searches for `alice`
        let req = test::TestRequest::get()
            .uri("/lime/my")
            .insert_header(("AUTH_TOKEN", auth_token))
            .insert_header(ContentType::json())
            .to_request();

        let resp: Value = test::call_and_read_body_json(&mut app, req).await;

        // Check the response to make sure correct list of transactions were returned
        let transactions: Vec<Transaction> = serde_json::from_value(resp["transactions"].clone()).expect("Failed to parse transactions");
        assert!(!transactions.is_empty(), "No transactions found for user");

        let transaction = transactions.first().unwrap();
        assert_eq!(transaction.transaction_hash, tx_hash);
    }
}
