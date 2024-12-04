
# Ethereum Fetcher

My solution of the take-home assignment from Limechain. This Rust server provides an API for fetching Ethereum transactions identified by their transaction hashes and their RLP Hex encoded representation.

## Architecture of the Server

### Overview

The server is built with Rust, leveraging the `actix-web` framework for creating RESTful services. The application interacts with a PostgreSQL database and is containerized using Docker.

The main components of the server include:

1. **Authentication Module**: Handles user authentication using JWT tokens.
2. **Transactions Module**: Manages Ethereum transactions, including fetching, decoding, and storing them in a PostgreSQL database.
3. **Routes**: Defines endpoints for interacting with the server, including fetching transactions and user-specific queries.
4. **Database Integration**: Utilizes Diesel ORM for database operations. (Automatic migrations, ensuring the correct tables and relations are set up).

### Endpoints
- **`/lime/eth?transactionHashes=...`**: Fetches Ethereum transactions based on transaction hashes.
- **`/lime/eth/{rlphex}`**: Decodes RLP hex strings to fetch Ethereum transactions.
- **`/lime/my`**: Retrieves transactions queried by the authenticated user.
- **`/lime/authenticate`**: Authenticates users and provides JWT tokens.

### Database
- PostgreSQL is used to store transaction data and user search history.

### Environment Variables
- Configures database connections and external Ethereum node URLs. (Most importantly)
    - `DB_CONNECTION_URL`
    - `ETH_NODE_URL`
    - `API_PORT`
    - `JWT_SECRET`

## How to Run the Server

### Prerequisites
- PostgreSQL installed: [PostgreSQL Installation Guide](https://www.postgresql.org/download/)
- Rust installed: [Rust Installation Guide](https://www.rust-lang.org/tools/install)
- Docker installed: [Docker Installation Guide](https://docs.docker.com/engine/install/)
  - Docker image of Postgres *(optional)*: `docker pull postgres`

### Steps (Separately Starting the Database and Server)

1. **Check Port 5432**: Ensure no service is running on port 5432.
   ```sh
   sudo lsof -i -P -n | grep 5432
   ```
   To stop existing services:
   ```sh
   sudo service postgresql stop
   # or
   docker stop <container_id>
   docker rm <container_id>
   ```

2. **Create Network**: Create a network for the database and server containers.
   ```sh
   docker network create limeapi-network
   ```

3. **Run Database Container**: Start a PostgreSQL container.
   ```sh
   docker run --name limeapi-db \
     --network limeapi-network \
     -e POSTGRES_USER=admin \
     -e POSTGRES_PASSWORD=1234 \
     -e POSTGRES_DB=postgres \
     -p 5432:5432 \
     -d postgres
   ```

4. **Build Server Image**: Build the Docker image for the server.
   ```sh
   docker build -t limeapi .
   ```

5. **Run Server Container**: Start the server container with the appropriate environment variables.
   ```sh
   docker run -p 8080:8080 \
     --network limeapi-network \
     -e API_PORT=8080 \
     -e PG_DBNAME=postgres \
     -e PG_HOST=limeapi-db \
     -e DB_CONNECTION_URL='postgresql://admin:1234@limeapi-db:5432/postgres' \
     -e ETH_NODE_URL='your_eth_node_url' \
     -e JWT_SECRET=your_jwt_secret \
     limeapi
   ```

### Steps (for the "*Easy Setup*")

1. **Edit `compose.yaml`**: Edit the configuration file with your `ETH_NODE_URL`.

2. **Run the Server and Database**:
   ```sh
   docker compose up --build -d
   ```

### Running Integration Tests

#### Prerequisites
- Ensure port `5432` is free from any existing PostgreSQL service.

#### Steps

1. **Run Test Database Container**: Start a PostgreSQL container for testing.
   ```sh
   docker run --name test-postgres -e POSTGRES_USER=admin -e POSTGRES_PASSWORD=1234 -e POSTGRES_DB=test_db -p 5432:5432 -d postgres
   ```

2. **Run Tests**: Execute the integration tests.
   ```sh
   DB_CONNECTION_URL='postgresql://admin:1234@localhost:5432/test_db' ETH_NODE_URL='your_eth_node_url' cargo test
   ```

## Requests and Responses

### `/lime/eth?transactionHashes`

- **Request**: `GET /lime/eth?transactionHashes=<hash1>&transactionHashes=<hash2>...`
- *(Optional) Header*: `AUTH_TOKEN: <token>`
- **Response**:
  ```json
  {
      "transactions": [
          {
              "transactionHash": "0x...",
              "transactionStatus": 1,
              "blockHash": "0x...",
              "blockNumber": 5703601,
              "from": "0x...",
              "to": "0x...",
              "contractAddress": null,
              "logsCount": 0,
              "input": "0x...",
              "value": "500000000000000000"
          }
      ]
  }
  ```
- **Example** (using `curl`):
  ```sh
  curl -X GET "localhost:8080/lime/eth?transactionHashes=0x4bdbf80e6fc6128de296d6fe06180240bf9bf8d603d13ce8ef59a599f5afc432"
  ```

  ```sh
  curl -X GET -H 'Content-Type: application/json' -H 'AUTH_TOKEN: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VybmFtZSI6ImFsaWNlIiwiZXhwIjoxNzE5Nzk0Mzg4fQ.VjVSpSSbQvFzn009yJCBEL-ZMHSsLwZ-mMhIjdmKJ9Q' -i 'localhost:8080/lime/eth?transactionHashes=0xfc2b3b6db38a51db3b9cb95de29b719de8deb99630626e4b4b99df056ffb7f2e&transactionHashes=0x48603f7adff7fbfc2a10b22a6710331ee68f2e4d1cd73a584d57c8821df79356&transactionHashes=0xcbc920e7bb89cbcb540a469a16226bf1057825283ab8eac3f45d00811eef8a64&transactionHashes=0x6d604ffc644a282fca8cb8e778e1e3f8245d8bd1d49326e3016a3c878ba0cbbd'
  ```

### `/lime/eth/{rlphex}`

- **Request**: `GET /lime/eth/{rlphex}`
- *(Optional) Header*: `AUTH_TOKEN: <token>`
- **Response**:
  ```json
  {
      "transactions": [
          {
              "transactionHash": "0x...",
              "transactionStatus": 1,
              "blockHash": "0x...",
              "blockNumber": 5703601,
              "from": "0x...",
              "to": "0x...",
              "contractAddress": null,
              "logsCount": 0,
              "input": "0x...",
              "value": "500000000000000000"
          }
      ]
  }
  ```
- **Examples** (using `curl`):
  ```sh
  curl -X GET 'localhost:8080/lime/eth/0xf842a071d6d42dfa97d9a5b9c7db21844e0139c594f35cd2ce3cd71be22990b9d2b58da0d2b59543277ba85c6e9db5ec9da836ea796af469c9ff5241b734525b6721e242'
  ```

  ```sh
  curl -X GET -H 'Content-Type: application/json' -H 'AUTH_TOKEN: <token>' -i 'localhost:8080/lime/eth/0xf842a071d6d42dfa97d9a5b9c7db21844e0139c594f35cd2ce3cd71be22990b9d2b58da0d2b59543277ba85c6e9db5ec9da836ea796af469c9ff5241b734525b6721e242'
  ```

### `/lime/my`

- **Request**: `GET /lime/my`
  - **Header**: `AUTH_TOKEN: <token>`
- **Response**:
  ```json
  {
      "transactions": [
          {
              "transactionHash": "0x...",
              "transactionStatus": 1,
              "blockHash": "0x...",
              "blockNumber": 5703601,
              "from": "0x...",
              "to": "0x...",
              "contractAddress": null,
              "logsCount": 0,
              "input": "0x...",
              "value": "500000000000000000"
          }
      ]
  }
  ```
- **Examples** (using `curl`):
  ```sh
  curl -X GET -H 'Content-Type: application/json' -H 'AUTH_TOKEN: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VybmFtZSI6ImFsaWNlIiwiZXhwIjoxNzE5Nzk0Mzg4fQ.VjVSpSSbQvFzn009yJCBEL-ZMHSsLwZ-mMhIjdmKJ9Q' -i 'localhost:8080/lime/my'
  ```

  ```sh
  curl -X GET -H 'Content-Type: application/json' -H 'AUTH_TOKEN: <token>' -i 'localhost:8080/lime/my'
  ```

### `/lime/authenticate`

- **Request**: `POST /lime/authenticate`
  - **Body**:
    ```json
    {
        "username": "alice",
        "password": "alice"
    }
    ```
- **Response**:
  ```json
  {
      "token": "<jwt_token>"
  }
  ```
- **Examples** (using `curl`):
  ```sh
  curl -X POST -H 'Content-Type: application/json' -i 'localhost:8080/lime/authenticate' --data '{"username":"alice", "password":"alice"}'
  ```
