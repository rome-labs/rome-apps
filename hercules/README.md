# Hercules

Hercules is an application designed to index Rome-EVM (Ethereum Virtual Machine) transactions on the Solana blockchain.

## Internal structure
On the higher level, Hercules consists of several main components. These are:

### 2. Solana Block Loader (optional)
Reads Solana blocks using Solana RPC, filters out Rome-EVM transactions, and stores data into Solana Block Storage.
This component is optional and MUST be omitted in case if using Solana Block Storage in Relayer mode.
It is possible also to omit Solana Block Loader if you use Solana blocks tables constantly replicated from another 
instance of Postgres DB where blocks are available. 

### 3. Solana Block Storage
Stores solana blocks persistently. There are 2 implementations of Solana Block Storage:
  - PgStorage - stores all Solana blocks in PostgreSQL DB
  - Relayer - reads Solana blocks from Relayer RPC API

### 4. Rollup Indexer
Central component providing integration between Solana and Ethereum representation of data. Includes logic of retries 
and error handling

### 5. Block Parser
Main part of Rollup indexer.
Sequentially parses Solana blocks and transactions retrieved by Rollup Indexer from Solana Block Storage, 
converts it into a form compatible with Ethereum API for it to be later stored into Ethereum Block Storage

### 6. Ethereum Block Storage
Stores Rome-EVM blocks and transactions in a persistent manner. Currently, one implementation of Ethereum Block
Storage is available - this implementation uses PostgreSQL DB as a storage backend

### 7. Block Producer (optional)
Component calculating block hashes and block numbers for Rome-EVM blocks. In a case of op-geth integration (read below),
Block Producer is using Engine API of op-geth node to retrieve this parameters. After production, this information is 
written back to Ethereum Block Storage

### 8. Admin API
JSON RPC API used to provide and access to additional functions of Hercules service and monitor its state.

- #### inSync()
is indexer synchronized with latest solana block? true/fase 

- #### lastSolanaStorageSlot()
returns number of last Solana slot in Solana Block Storage or Null in case if there is no slots

- #### lastEthereumStorageSlot()
returns the number of last Solana slot containing produced eth-locks or Null in case of there is no produced eth-blocks

#### Optional methods
Admin API gets two additional API methods in case when Block Producer is not included into configuration. These methods 
allow to integrate Hercules with external block producer (op-node in case of Based Rollup Sequencer setup)

- #### getPendingBlocks()
returns JSON structure described by Rust type 
[ProducerParams](https://github.com/rome-labs/rome-sdk/blob/main/rome-evm-client/src/indexer/ethereum_block_storage.rs#L27) 
defined in Rome SDK of null if there's no pending blocks.

- #### blocksProduced(produced_blocks)
Receives produced blocks from external block producer and stores this information into Ethereum Block Storage.
produced_blocks is a JSON structure described by Rust type [ProducedBlocks](https://github.com/rome-labs/rome-sdk/blob/main/rome-evm-client/src/indexer/produced_blocks.rs)
defined in Rome SDK


## Hercules internals

![A descriptive alt text](./common-schema.png)

## Configuration
Hercules is requiring environment variable HERCULES_CONFIG to be specified in the environment. This variable must point
to the file containing configuration parameters of the service in an YAML/JSON format. Below is the description of each
section and parameters of this configuration file:

- **block_loader**

  (optional) Parameters of **Solana Block Loader**
  - **program_id** - base58 address of the Rome-EVM smart-contract on Solana
  - **batch_size** - how many blocks should Solana Block Loader load in parallel when indexing. This
    value may increase loading speed on a good network connection and increase number of error if the connection is bad
  - **block_retries** - how many times should Solana Block Loader retry to load a block if it fails.
  - **tx_retries** - how many times should Solana Block Loader retry to load a transaction if it fails.
  - **retry_int_sec** - how long should Solana Block Loader wait (in seconds) before retrying to load a block or transaction.
  - **commitment** - default commitment level to send requests with. Possible values: **Confirmed, Finalized**
  - **client** - configuration of the Solana Client.
    - **providers** - array of URLs of Solana RPC nodes.


- **solana_storage**

  Parameters of Solana Block Storage
  - **type** - type of the backend. Currently available: **pg_storage**
  
  #### type: pg_storage - PostgreSQL backend for Solana Block Storage
  - **connection**
    - **database_url** - connection string in diesel-compatible format: **postgres://\<username\>:\<password\>@\<server\>/\<database\>**
    - **max_connections** - number of parallel connections for the connection pool
    - **connection_timeout_sec** - connection timeout in seconds
  
  ### type: relayer - Relayer backend for Solana Block Storage
  - **relayer_url** - URL of Relayer RPC API.
  - **connection**
    - **database_url** - connection string in diesel-compatible format: **postgres://\<username\>:\<password\>@\<server\>/\<database\>**
    - **max_connections** - number of parallel connections for the connection pool
    - **connection_timeout_sec** - connection timeout in seconds


- **block_parser**
  
  Parameters of **Block Parser**
  - **program_id** - (optional - program_id from block_loader config will be used if absent) base58 address of the Rome-EVM smart-contract on Solana
  - **chain_id** - chain id withing selected Rome-EVM smart-contract
  - **parse_mode** - determines algorithm of block parsing. Can accept values
    - **engine_api** - parsing compatible with Engine API protocol - sequential Rome-EVM transactions with the same gas 
      recipient will be packed into separate eth-block. One solana block can be converted into several eth-blocks
    - **single_state** - parsing for single-state schema. All the transactions from particular Solana block are packed 
      into single eth-block
  

- **ethereum_storage**

  Parameters of **Ethereum Block Storage**
  - **type** - type of the backend. Currently available: **pg_storage**
  
  #### type: pg_storage - PostgreSQL backend for Ethereum Block Storage
  - **connection**
    - **database_url** - connection string in diesel-compatible format: **postgres://\<username\>:\<password\>@\<server\>/\<database\>**
    - **max_connections** - number of parallel connections for the connection pool
    - **connection_timeout_sec** - connection timeout in seconds


- **block_producer**

  (Optional) parameters of **Block Producer**. **Admin API** will provide additional methods for block production in case if
  **block_producer** section is absent in configuration
  - **type** - type of the Block Producer. Currently available: **engine_api, single_state**
  
    **NOTE: block_parser.parse_mode = block_producer.type - THIS IS MANDATORY** 

  - **engine_api** - Engine API Block Producer using op-geth for block production
    - **geth_engine** parameters of Engine API connection
      - **geth_engine_addr** - URL of Engine API RPC (usually resides on op-geth port 8551)
      - **geth_engine_secret** - Engine API authentication token
    - **geth_api** - URL of op-geth Ethereum API (usually resides on op-geth port 8545)

  - **single_state** - Single state block producer - copies Solana block parameters to eth-blocks


- **start_slot** - number of Solana slot to start indexation at
- **admin_rpc** - where to expose Admin API. Accepts string of a format: <IPv4_ADDRESS>:<PORT_NUMBER>
- **max_slot_history (optional)** - how many Solana blocks to store in the Solana Block Storage - all in case if not specified
- **mode** - mode of operation. Possible values are: **Indexer** - normal indexation mode, **Recovery** - recover solana block history (indexation is disabled, Admin API disabled).

## Supported Configurations
Hercules can participate in several different Rome-EVM setups depending on the needs: 
- Rome-EVM Rollup on Solana (L1) with op-geth client
- Rome-EVM Based Rollup on Ethereum (L1)
- Rome-EVM Rollup on Solana (L1) with a Custom rome Client

Service should be properly configured to function in each of these setups. Below is an example configurations for these
use-cases

### 1. Rome-EVM Rollup on Solana (L1) with op-geth client
Solana serves as the L1, hosting the Rome-EVM rollup. 
Hercules indexes Rome-EVM transactions from Solana blocks and advances state of op-geth client. User interacts with the
rollup over op-geth client.

### Example:

```yml
block_loader:
  program_id: "CmobH2vR6aUtQ8x4xd1LYNiH6k2G7PFT5StTgWqvy2VU"
  batch_size: 64
  block_retries: 10
  tx_retries: 100
  retry_int_sec: 1
  commitment: "confirmed"
  client:
    providers:
      - "http://solana1:8899"
      - "http://solana2:8899"
      - "http://solana3:8899"
solana_storage:
  type: pg_storage
  connection:
    database_url: "postgres://hercules:qwerty123@postgres/test_rollup"
    max_connections: 16
    connection_timeout_sec: 30
block_parser:
  program_id: "CmobH2vR6aUtQ8x4xd1LYNiH6k2G7PFT5StTgWqvy2VU"
  chain_id: 1001
  parse_mode: engine_api
ethereum_storage:
  type: pg_storage
  connection:
    database_url: "postgres://hercules:qwerty123@postgres/test_rollup"
    max_connections: 16
    connection_timeout_sec: 30
block_producer:
  type: engine_api
  geth_engine:
    geth_engine_addr: "http://geth:8551"
    geth_engine_secret: "a535c9f4f9df8e00cd6a15a7baa74bb92ca47ebdf59b6f3f2d8a8324b6c1767c"
  geth_api: "http://geth:8545"
start_slot: 0
admin_rpc: "0.0.0.0:8000"
max_slot_history: 4096
mode: Indexer
```

### 2. Rome-EVM Based Rollup on Ethereum (L1)
Ethereum plays a role of consensus layer, the Rome-EVM on Solana provides execution layer. Hercules, indexes Rome-EVM 
transactions from Solana blocks and provides pre-confirm data for op-node. User interacts with the
rollup over op-geth client. In this configuration, Block Producer is disabled in Hercules, because blocks are produced
by op-node. Admin API contains additional methods - *getPendingBlocks* and *blocksProduced* which is used by op-node
during block production process

### Example:

```yml
block_loader:
  program_id: "CmobH2vR6aUtQ8x4xd1LYNiH6k2G7PFT5StTgWqvy2VU"
  batch_size: 64
  block_retries: 10
  tx_retries: 100
  retry_int_sec: 1
  commitment: "confirmed"
  client:
    providers:
      - "http://solana1:8899"
      - "http://solana2:8899"
      - "http://solana3:8899"
solana_storage:
  type: pg_storage
  connection:
    database_url: "postgres://hercules:qwerty123@postgres/test_rollup"
    max_connections: 16
    connection_timeout_sec: 30
block_parser:
  program_id: "CmobH2vR6aUtQ8x4xd1LYNiH6k2G7PFT5StTgWqvy2VU"
  chain_id: 1001
  parse_mode: engine_api
ethereum_storage:
  type: pg_storage
  connection:
    database_url: "postgres://hercules:qwerty123@postgres/test_rollup"
    max_connections: 16
    connection_timeout_sec: 30
start_slot: 0
admin_rpc: "0.0.0.0:8000"
max_slot_history: 4096
mode: Indexer
```

### 3. Rome-EVM Rollup on Solana (L1) with a Custom rome Client
Solana serves as the L1, hosting the Rome-EVM rollup. Hercules indexes Rome-EVM transactions from Solana blocks and 
prepares transaction and block history data for custom Rome-client. This schema also known as "Single state".
User interacts with the rollup over custom Rome-client.

```yml
block_loader:
  program_id: "CmobH2vR6aUtQ8x4xd1LYNiH6k2G7PFT5StTgWqvy2VU"
  batch_size: 64
  block_retries: 10
  tx_retries: 100
  retry_int_sec: 1
  commitment: "confirmed"
  client:
    providers:
      - "http://solana1:8899"
      - "http://solana2:8899"
      - "http://solana3:8899"
solana_storage:
  type: pg_storage
  connection:
    database_url: "postgres://hercules:qwe~~~~rty123@postgres/test_rollup"
    max_connections: 16
    connection_timeout_sec: 30
block_parser:
  program_id: "CmobH2vR6aUtQ8x4xd1LYNiH6k2G7PFT5StTgWqvy2VU"
  chain_id: 1001
  parse_mode: single_state
ethereum_storage:
  type: pg_storage
  connection:
    database_url: "postgres://hercules:qwerty123@postgres/test_rollup"
    max_connections: 16
    connection_timeout_sec: 30
block_producer:
  type: single_state
start_slot: 0
admin_rpc: "0.0.0.0:8000"
max_slot_history: 4096
mode: Indexer
```



## Tracing on Otel Telemetry and Logging 

Rhea supports both OpenTelemetry-based tracing and standard output logging.

Tracing is configured via the following environment variables:

- `ENABLE_OTEL_TRACING`: Enables OpenTelemetry tracing. Set this to `true` to activate it.
- `ENABLE_OTEL_METRICS`: Enables OpenTelemetry metrics collection. Set this to `true` to collect metrics.
- `OTLP_RECEIVER_URL`: Specifies the OTLP (OpenTelemetry Protocol) receiver endpoint. 

By default, standard output logging is also enabled. You can toggle this behavior using:

- `ENABLE_STDOUT_LOGGING_ENV`: Set to `false` to disable stdout logging if needed.

The proxy will emit tracing data in JSON format to stdout unless overridden, and will export spans and metrics to the configured OTLP receiver if tracing and metrics are enabled.

### Example Docker Compose Configuration
```yaml
environment:
  ENABLE_OTEL_TRACING: true
  ENABLE_OTEL_METRICS: true
  OTLP_RECEIVER_URL: http://localhost:4317
  ENABLE_STDOUT_LOGGING_ENV: true
```

This setup ensures that all service-level telemetry is collected and available for observability tooling such as Jaeger, Prometheus, or Grafana via the OTLP pipeline.

