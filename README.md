# Blockchain in Rust

The blockchain algorithm from the Bitcoin white paper (implemented in Rust).

## Features

The following features are completed:

1. The ability to create wallets, transactions, and blocks, and add them to the
   blockchain.
2. The proof-of-work algorithm for mining a block to add to the blockchain (a
   simplified version of the one described in the bitcoin whitepaper)
3. The Merkle-tree data-structure for verifying transactions.
4. Digital-signatures for Wallets to lock and unlock transaction outputs.
5. The network-layer implemented using the actor-model for simulation on a
   single machine.

## Running Instructions

Build and run the project with the following commands:

```shell
$ cargo build
```

```shell
$ cargo run
```

## Digital Signatures

Each created wallet has a public key - private key pair generated based on the
[Elliptic Curve Digital Signature Algorithm or
ECDSA](https://en.bitcoin.it/wiki/Elliptic_Curve_Digital_Signature_Algorithm)
algorithm. The public key is hashed using the SHA256 algorithm and the RIPEMD-160
algorithm in succession. The address of the wallet is the concatenation of the
version, the public-key hash, and the checksum of the public-key hash (which is
obtained by passing the public-key hash through the SHA256 algorithm twice).

```
Public Key -> SHA256 -> RIPEMD-160 -> Public Key Hash
Public Key Hash -> SHA256 -> SHA256 -> Checksum
address = VERSION + Public Key Hash + Checksum
```

## Simulation

The network layer is simulated using the Actix framework to spawn multiple
containers (nodes) on the same machine. The actor model also provides the
flexibility of allowing actors to spawn in different machines also communicate with
each other.

The main program allows spawning multiple nodes, creating a blockchain on one
node, and having it distributed via messages.

The following are the different types of messages that nodes use to communicate.

1. Sent from the main program to a node(actor):

- `CreateBlockchain`: Instruct the node to start a blockchain (create a genesis
  block with a single coinbase transaction and mine it)
- `UpdateBlockchainFromKnownNodes`: If a node wishes to purge its copy of the
  blockchain, or has newly joined the network, it can request a copy from its
  neighbors.

2. Sent from the main program or another node(actor)

- `UpdateRoutingInfo`: Instruct the node to update its neighbor list with the
  address list sent in the payload.
- `AddTransactionAndMine`: Instruct the node to add a transaction to its block
  and mine it. After mining, the block is added to its copy of the blockchain,
  and the added block is sent to its neighbors for replication.

3. Sent from another node(actor)

- `RequestBlockchain`: TODO
- `Blockchain`: A copy of the sender's blockchain.
- `Block`: A block (using mined by the sender).

## To-Do

- Use the Merkle-tree to verify transactions.
- Implement persistent storage for unspent transactions.
- Choose the network topology to simulate (right now, all simulations assume the
  network is full)
