use std::{str::FromStr};

use actix_web::{web, get, Responder, HttpResponse, HttpRequest};
use actix_web_lab::extract::Query;

use ethers::types::H256;

use crate::transaction::{
    Transaction,
    TransactionHashes,
    DecodeError,
    store_transaction_in_db,
    get_transaction_from_db,
    get_all_transactions_from_db,
    fetch_transaction,
    decode_rlp_hex,
};
use crate::users::{
    get_user_from_token,
    store_user_search,
    get_user_search_transactions,
};
use crate::DBPool;

#[get("/lime/eth")]
async fn lime_eth_transactions_hashes(query: Query<TransactionHashes>, pool: web::Data<DBPool>, req: HttpRequest) -> impl Responder {
    let TransactionHashes { hashes } = query.into_inner();
    let user = get_user_from_token(&req).await;

    let mut transactions : Vec<Transaction> = Vec::new();

    for hash_str in &hashes {
        let mut transaction_found = true;

        if let Ok(Some(tx)) = get_transaction_from_db(&pool, hash_str).await {
            transaction_found = true;
            transactions.push(tx);
        } else {
            if let Ok(tx_hash) = H256::from_str(hash_str) {
                if let Some(tx) = fetch_transaction(tx_hash).await {
                    if let Err(e) = store_transaction_in_db(pool.clone(), &tx).await {
                        // Shouldn't happen!!
                        eprintln!("Failed to save transaction: {}", e);
                    }
                    transaction_found = true;
                    transactions.push(tx);
                }
            }
            else {
                return HttpResponse::BadRequest().body("Invalid Transaction Hash provided!");
            }
        }

        if transaction_found {
            if let Some(ref user) = user {
                if let Err(e) = store_user_search(pool.clone(), user, hash_str).await {
                    eprintln!("Failed to save user search: {}", e);
                }
            }
        }
    }

    let response = serde_json::json!({ "transactions": transactions });

    HttpResponse::Ok().json(response)
}

#[get("/lime/all")]
pub async fn lime_all(pool: web::Data<DBPool>) -> impl Responder {
    match get_all_transactions_from_db(&pool).await {
        Ok(transactions) => {
            let response = serde_json::json!({ "transactions": transactions });
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Failed to fetch transactions: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch transactions")
        }
    }
}

#[get("/lime/eth/{rlphex}")]
pub async fn lime_eth_rlphex(path: web::Path<String>, pool: web::Data<DBPool>, req: HttpRequest) -> impl Responder {
    let user = get_user_from_token(&req).await;

    let rlp_hex = path.into_inner();
    let hashes = match decode_rlp_hex(&rlp_hex) {
        Ok(hashes) => hashes,
        Err(DecodeError::InvalidHex) => return HttpResponse::BadRequest().body("Invalid Hex String"),
        Err(DecodeError::InvalidHashLen) => return HttpResponse::BadRequest().body("Invalid Hash Length"),
    };

    let mut transactions: Vec<Transaction> = Vec::new();
    for hash in hashes {
        let mut transaction_found = false;
        let tx_hash_str = format!("{:?}", hash);
        
        if let Ok(Some(tx)) = get_transaction_from_db(&pool, &tx_hash_str).await {
            transaction_found = true;
            transactions.push(tx);
        } else {
            if let Some(tx) = fetch_transaction(hash).await {
                if let Err(e) = store_transaction_in_db(pool.clone(), &tx).await {
                    // Shouldn't happen!!
                    eprintln!("Failed to save transaction: {}", e);
                }
                transaction_found = true;
                transactions.push(tx);
            }
        }

        if transaction_found {
            if let Some(ref user) = user {
                if let Err(e) = store_user_search(pool.clone(), user, &tx_hash_str).await {
                    eprintln!("Failed to save user search: {}", e);
                }
            }
        }

    }

    let response = serde_json::json!({ "transactions": transactions });
    HttpResponse::Ok().json(response)
}


#[get("/lime/my")]
pub async fn lime_my(req: HttpRequest, pool: web::Data<DBPool>) -> impl Responder {
    if let Some(username) = get_user_from_token(&req).await {
        match get_user_search_transactions(&pool, &username).await {
            Ok(transactions) => {
                let response = serde_json::json!({ "transactions": transactions });
                HttpResponse::Ok().json(response)
            }
            Err(e) => {
                eprintln!("Failed to fetch user transactions: {}", e);
                HttpResponse::InternalServerError().body("Failed to fetch user transactions")
            }
        }
    } else {
        HttpResponse::Unauthorized().body("Invalid or missing AUTH_TOKEN")
    }
}
