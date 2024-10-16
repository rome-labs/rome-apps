# Rome Apps

### Cli application for the rome-evm program

Usage: 

`cli --program-id <PROGRAM_ID> --chain-id <CHAIN_ID> --url <URL> <COMMAND>`

Commands:

* `reg-rollup`         registry a rollup in rome-evm contract
* `create-balance`     create balance on the address of the rollup owner; used to synchronize the initial state of rollup with the state of op-geth
* `reg-gas-recipient`  registry the operator's gas recipient account

Options:
* `-p, --program-id <PROGRAM_ID>`  rome-evm program_id
* `-c, --chain-id <CHAIN_ID>`      chain_id of rollup
* `-u, --url <URL>`                URL for Solana's JSON RPC: http://localhost:8899

##### Registry a rollup in rome-evm contract

`cli --program-id <PROGRAM_ID> --chain-id <CHAIN_ID> --url <URL> reg-rollup <ROLLUP_OWNER> <UPGRADE_AUTHORITY>`

* `<ROLLUP_OWNER>`       rollup owner Pubkey
* `<UPGRADE_AUTHORITY>`  path to upgrade-authority keypair of the rome-evm contract


##### Create balance on the address of the rollup owner

`cli --program-id <PROGRAM_ID> --chain-id <CHAIN_ID> --url <URL> create-balance <ADDRESS> <BALANCE> <KEYPAIR>`

* `<ADDRESS>`  the contract owner's address to mint a balance
* `<BALANCE>`  balance to mint
* `<KEYPAIR>`  path to rollup owner keypair

##### Registry the operator's gas recipient account

`cli --program-id <PROGRAM_ID> --chain-id <CHAIN_ID> --url <URL> reg-gas-recipient <ADDRESS> <KEYPAIR>`

* `<ADDRESS>`  the gas recipient address of the operator
* `<KEYPAIR>`  path to keypair of the operator


#### Example
`./cli --program-id CmobH2vR6aUtQ8x4xd1LYNiH6k2G7PFT5StTgWqvy2VU --chain-id 1001 --url http://localhost:8899 reg-rollup FvzoxsNHajMvErQmMsn9h8ndAXweo3vqn9gfEgAdpPka /opt/ci/upgrade-authority-keypair.json `

`./cli --program-id CmobH2vR6aUtQ8x4xd1LYNiH6k2G7PFT5StTgWqvy2VU --chain-id 1001 --url http://localhost:8899 create-balance 0xe235b9caf55b58863Ae955A372e49362b0f93726 1000 /opt/ci/rollup-owner-keypair.json `

`./cli --program-id CmobH2vR6aUtQ8x4xd1LYNiH6k2G7PFT5StTgWqvy2VU --chain-id 1001 --url http://localhost:8899 reg-gas-recipient 0x229E93198d584C397DFc40024d1A3dA10B73aB32  /opt/ci/proxy-sender.json `