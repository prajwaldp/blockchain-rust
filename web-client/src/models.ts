export interface Node {
  id: string; // node name
  messages: string[];
  neighbourCount: number;
  blockchain: Blockchain;
}

export interface Wallet {
  id: string; // wallet address
  balance: number;
}

export interface TransactionInput {
  id: string;
  out: number;
  signature: string;
  publicKey: string;
}

export interface TransactionOutput {
  value: number;
  publicKeyHash: string;
}

export interface Transaction {
  id: string;
  inputs: TransactionInput[];
  outputs: TransactionOutput[];
}

export interface Block {
  index: number;
  timestamp: number;
  hash: string;
  proofOfWork: number;
  nonce: number;
  transactions: Transaction[];
}

export interface Blockchain {
  length: number;
  blocks: Block[];
}
