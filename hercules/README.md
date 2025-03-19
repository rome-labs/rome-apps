# Hercules

Hercules is an application designed to index Rome-EVM (Ethereum Virtual Machine) transactions on the Solana blockchain.

## Internal structure
On the higher level, Hercules consists of several main components. These are:

### 1. Solana RPC Connector
Interacts with Solana RPC

### 2. Solana Block Loader
Reads Solana blocks using Solana RPC, filters out Rome-EVM transactions and stores data into Solana Block Storage

### 3. Solana Block Storage
Stores solana blocks and Rome-EVM transactions in a persistent manner. Currently, one implementation of Solana Block
Storage is available - this implementation uses PostgreSQL DB as a storage backend

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

- **solana**

    Parameters of **Solana RPC Connector**
  - **rpc_url** - URL of Solana RPC API
  - **commitment** - default commitment level to send requests with. Possible values: **confirmed, finalized**


- **solana_storage**

  Parameters of Solana Block Storage
  - **type** - type of the backend. Currently available: **pg_storage**
  
  #### type: pg_storage - PostgreSQL backend for Solana Block Storage
  - **connection**
    - **database_url** - connection string in diesel-compatible format: **postgres://\<username\>:\<password\>@\<server\>/\<database\>**
    - **max_connections** - number of parallel connections for the connection pool
    - **connection_timeout_sec** - connection timeout in seconds


- **block_parser**
  
  Parameters of **Block Parser**
  - **program_id** - base58 address of the Rome-EVM smart-contract on Solana
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
- **block_loader_batch_size** - how many blocks should Solana Block Loader to load in parallel when indexing. This
  value may increase loading speed on a good network connection and increase number of error if the connection is bad
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
solana:
  rpc_url: "http://solana:8899"
  commitment: "confirmed"
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
block_loader_batch_size: 128
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
solana:
  rpc_url: "http://solana:8899"
  commitment: "confirmed"
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
block_loader_batch_size: 128
mode: Indexer
```

### 3. Rome-EVM Rollup on Solana (L1) with a Custom rome Client
Solana serves as the L1, hosting the Rome-EVM rollup. Hercules indexes Rome-EVM transactions from Solana blocks and 
prepares transaction and block history data for custom Rome-client. This schema also known as "Single state".
User interacts with the rollup over custom Rome-client.

```yml
solana:
  rpc_url: "http://solana:8899"
  commitment: "confirmed"
solana_storage:
  type: pg_storage
  connection:
    database_url: "postgres://hercules:qwerty123@postgres/test_rollup"
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
block_loader_batch_size: 128
mode: Indexer
```
