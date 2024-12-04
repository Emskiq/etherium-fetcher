use actix_web::web;
use std::env;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;

use ethers::prelude::*;
use ethers::types::{Transaction as EthersTransaction, TransactionReceipt};
use ethers::utils::hex;

use rlp::Rlp;

use crate::DBPool;
use super::schema::transactions;

/// Transaction Hashes as Strings
#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionHashes {
    #[serde(rename = "transactionHashes")]
    pub hashes: Vec<String>,
}

/// Custom representation of transaction on the blockchain
#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[diesel(table_name = transactions)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub transaction_hash: String,
    pub transaction_status: bool,
    pub block_hash: String,
    pub block_number: i64,
    pub from: String,
    pub to: Option<String>,
    pub contract_address: Option<String>,
    pub logs_count: i64,
    pub input: String,
    pub value: String,
}

impl From<(EthersTransaction, TransactionReceipt)> for Transaction {
    fn from((tx, receipt): (EthersTransaction, TransactionReceipt)) -> Self {
        Self {
            transaction_hash: format!("{:?}", tx.hash),
            transaction_status: receipt.status.unwrap_or_default().as_u64() == 1,
            block_hash: format!("{:?}", receipt.block_hash.unwrap_or_default()),
            block_number: receipt.block_number.unwrap_or_default().as_u32() as i64,
            from: format!("{:?}", tx.from),
            to: tx.to.map(|to| format!("{:?}", to)),
            contract_address: receipt.contract_address.map(|addr| format!("{:?}", addr)),
            logs_count: receipt.logs.len() as i64,
            input: format!("{:?}", tx.input),
            value: format!("{:?}", tx.value),
        }
    }
}

pub async fn store_transaction_in_db(pool: web::Data<DBPool>, tx: &Transaction) -> Result<(), diesel::result::Error> {
    use crate::schema::transactions::dsl::*;

    let mut conn = pool.get().expect("couldn't get db connection from pool");

    // Check if the transaction already exists
    let existing_tx: Option<Transaction> = transactions
        .filter(transaction_hash.eq(&tx.transaction_hash))
        .first::<Transaction>(&mut conn)
        .optional()?;

    if existing_tx.is_none() {
        // Insert only if the transaction doesn't exist
        diesel::insert_into(transactions)
            .values(tx)
            .execute(&mut conn)?;
    }

    Ok(())
}

pub async fn get_transaction_from_db(pool: &DBPool, tx_hash: &str) -> Result<Option<Transaction>, diesel::result::Error> {
    use crate::schema::transactions::dsl::*;

    let mut conn = pool.get().expect("couldn't get db connection from pool");
    transactions.filter(transaction_hash.eq(tx_hash)).first::<Transaction>(&mut conn).optional()
}

pub async fn get_all_transactions_from_db(pool: &DBPool) -> Result<Vec<Transaction>, diesel::result::Error> {
    use crate::schema::transactions::dsl::*;

    let mut conn = pool.get().expect("couldn't get db connection from pool");
    transactions.load::<Transaction>(&mut conn)
}

pub async fn fetch_transaction(tx_hash: H256) -> Option<Transaction> {
    let eth_rpc_url = env::var("ETH_NODE_URL").unwrap_or_else(|_| "ETH_URL".to_string());
    let provider = ethers::providers::Provider::<ethers::providers::Http>::try_from(eth_rpc_url).unwrap();

    if let Ok(Some(tx)) = provider.get_transaction(tx_hash).await {
        if let Ok(Some(receipt)) = provider.get_transaction_receipt(tx_hash).await {
            return Some(Transaction::from((tx, receipt)));
        }
    }
    None
}

#[derive(Debug)]
pub enum DecodeError {
    InvalidHex,
    InvalidHashLen,
}

pub fn decode_rlp_hex(rlp_hex: &str) -> Result<Vec<H256>, DecodeError> {
    let rlp_data = match hex::decode(rlp_hex) {
        Ok(data) => data,
        Err(_) => return Err(DecodeError::InvalidHex),
    };

    let rlp = Rlp::new(&rlp_data).as_list::<Vec<u8>>();

    let mut hashes: Vec<H256> = Vec::new();
    for hash_bytes in rlp {
        if hash_bytes.len() != 32 {
            return Err(DecodeError::InvalidHashLen);
        }
        hashes.push(H256::from_slice(&hash_bytes));
    }

    Ok(hashes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_rlp_hex_valid() {
        let rlp_hex = "0xf842a06d61b62233334ebfb28515b4e2aa0e1011fdf542cf4948ef2831b65f0f1fe542a02f5bf15391119b4851c619f97d15ec9c0bd580eb034c29ce32e4c35bb3f288eb";
        let expected_hashes = vec![
            H256::from_slice(&hex::decode("6d61b62233334ebfb28515b4e2aa0e1011fdf542cf4948ef2831b65f0f1fe542").unwrap()),
            H256::from_slice(&hex::decode("2f5bf15391119b4851c619f97d15ec9c0bd580eb034c29ce32e4c35bb3f288eb").unwrap()),
        ];
        let decoded = decode_rlp_hex(rlp_hex).unwrap();
        assert_eq!(decoded, expected_hashes);
    }

    #[test]
    fn test_decode_rlp_hex_invalid_hex() {
        let rlp_hex = "zzzz";
        let decoded = decode_rlp_hex(rlp_hex);
        assert!(matches!(decoded, Err(DecodeError::InvalidHex)));
    }

    #[test]
    fn test_decode_rlp_hex_invalid_hash_len() {
        let rlp_hex = "0xe1a06d61b62233334ebfb12311313131313111fdf542cf4948ef2831b65f0f1fe542231331";
        let decoded = decode_rlp_hex(rlp_hex);
        assert!(matches!(decoded, Err(DecodeError::InvalidHashLen)));
    }
}
