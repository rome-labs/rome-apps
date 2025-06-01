# Rome Apps

### Cli application for the rome-evm program

Usage: 

`cli --program-id <PROGRAM_ID> --chain-id <CHAIN_ID> --url <URL> <COMMAND>`

Commands:

*  `reg-rollup`             registry a rollup in rome-evm contract
*  `create-balance`         create balance on the address of the rollup owner; used to synchronize the initial state of rollup with the state of op-geth
*  `get-balance`            get balance
*  `get-code`               get contract code
*  `get-storage-at`         get storage slot
*  `get-transaction-count`  get transaction count
*  `get-rollups`            get list of registered rollups
*  `help`                   Print this message or the help of the given subcommand(s)

Options:
* `-p, --program-id <PROGRAM_ID>`  rome-evm program_id
* `-c, --chain-id <CHAIN_ID>`      chain_id of rollup
* `-u, --url <URL>`                URL for Solana's JSON RPC: http://localhost:8899

##### Registry a rollup in rome-evm contract

`cli --program-id <PROGRAM_ID> --chain-id <CHAIN_ID> --url <URL> reg-rollup <UPGRADE_AUTHORITY>`

* `<UPGRADE_AUTHORITY>`  path to upgrade-authority keypair of the rome-evm contract


##### Deposit funds to the address

`cli --program-id <PROGRAM_ID> --chain-id <CHAIN_ID> --url <URL> deposit <ADDRESS> <BALANCE> <KEYPAIR>`

* `<ADDRESS>`  the contract owner's address to mint a balance
* `<BALANCE>`  balance in Wei to mint, must be multiple of 10^9
* `<KEYPAIR>`  path to user's solana wallet keypair; the funds will be debited from this account (lamports = balance/10^9) 


##### Get balance

`cli --program-id <PROGRAM_ID> --chain-id <CHAIN_ID> --url <URL> get-balance <ADDRESS>`

* `<ADDRESS>`  address to get balance

##### Get contract code

`cli --program-id <PROGRAM_ID> --chain-id <CHAIN_ID> --url <URL> get-code <ADDRESS>`

* `<ADDRESS>`  contract address
* 

##### Get storage slot

`cli --program-id <PROGRAM_ID> --chain-id <CHAIN_ID> --url <URL> get-storage-at <ADDRESS> <SLOT>`

* `<ADDRESS>`  contract address
* `<SLOT>`     slot

##### Get transaction count

`cli --program-id <PROGRAM_ID> --chain-id <CHAIN_ID> --url <URL> get-code <ADDRESS>`

* `<ADDRESS>`  contract address
*

##### Get list of registered rollups

`cli --program-id <PROGRAM_ID> --url <URL> get-rollups`

* `<PAYER>`  Solana payer pubkey
*

#### Example
`./cli --program-id CaQC27sVhdPyZF7defivoTQ48E8ws4tXvJfXYPRXboaH --chain-id 1001 --url http://localhost:8899 reg-rollup /opt/ci/upgrade-authority-keypair.json`

`./cli --program-id CaQC27sVhdPyZF7defivoTQ48E8ws4tXvJfXYPRXboaH --chain-id 1001 --url http://localhost:8899 deposit 0xe235b9caf55b58863Ae955A372e49362b0f93726 1000000000000000000 /opt/ci/test-account-keypair.json`

`./cli --program-id CaQC27sVhdPyZF7defivoTQ48E8ws4tXvJfXYPRXboaH --chain-id 1001 --url http://localhost:8899 get-balance 0x229E93198d584C397DFc40024d1A3dA10B73aB32`

`./cli --program-id CaQC27sVhdPyZF7defivoTQ48E8ws4tXvJfXYPRXboaH --chain-id 1001 --url http://localhost:8899 get-code 0x229E93198d584C397DFc40024d1A3dA10B73aB32`

`./cli --program-id CaQC27sVhdPyZF7defivoTQ48E8ws4tXvJfXYPRXboaH --chain-id 1001 --url http://localhost:8899 get-storage-at 0x229E93198d584C397DFc40024d1A3dA10B73aB32 0`

`./cli --program-id CaQC27sVhdPyZF7defivoTQ48E8ws4tXvJfXYPRXboaH --url http://localhost:8899 get-rollups`
