use actix_web::{web, HttpRequest};
use diesel::prelude::*;
use diesel::dsl::exists;
use diesel::select;

use crate::DBPool;
use crate::auth::verify_jwt;
use crate::schema::users_searches;
use crate::transaction::Transaction;

#[derive(Debug, Insertable)]
#[diesel(table_name = users_searches)]
pub struct UserSearch {
    pub username: String,
    pub transaction_hash: String,
}


// Not needed to be async actually
pub async fn get_user_from_token(req: &HttpRequest) -> Option<String> {
    if let Some(header) = req.headers().get("AUTH_TOKEN") {
        if let Ok(token) = header.to_str() {
            let claim_result = verify_jwt(token);
            match claim_result  {
                Ok(claim) => {
                    return Some(claim.username);
                }
                Err(_e) => {
                    return None;
                }
            }
        }
    }
    None
}

pub async fn store_user_search(pool: web::Data<DBPool>, user_searching: &str , transaction_hash_search: &str) -> Result<(), diesel::result::Error> {

    use crate::schema::users_searches::dsl::*;
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let user_search_exists: bool = select(exists(
            users_searches
                .filter(username.eq(&user_searching))
                .filter(transaction_hash.eq(&transaction_hash_search))))
        .get_result(&mut conn)?;

    if !user_search_exists {
        let new_search = UserSearch {
            username: user_searching.to_string(),
            transaction_hash: transaction_hash_search.to_string(),
        };


        diesel::insert_into(users_searches)
            .values(&new_search)
            .execute(&mut conn)?;
    }

    Ok(())
}

pub async fn get_user_search_transactions(pool: &web::Data<DBPool>, username: &str) -> Result<Vec<Transaction>, diesel::result::Error> {
    use crate::schema::transactions::dsl::{transactions as transactions_table, transaction_hash as transactions_hash};
    use crate::schema::users_searches::dsl::{users_searches as users_searches_table, transaction_hash as user_search_transaction_hash};

    let mut conn = pool.get().expect("couldn't get db connection from pool");

    // Fetch all transaction hashes for the user
    let user_searches = users_searches_table
        .filter(users_searches::username.eq(username))
        .select(user_search_transaction_hash)
        .distinct()
        .load::<String>(&mut conn)?;

    // Fetch all transactions for the user's searches
    let transactions = transactions_table
        .filter(transactions_hash.eq_any(user_searches))
        .load::<Transaction>(&mut conn)?;

    Ok(transactions)
}
