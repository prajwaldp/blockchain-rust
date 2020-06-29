import {
  TransactionInput,
  TransactionOutput,
  Transaction,
  Block,
  Blockchain,
} from "./models";

function toHexString(byteArray: Array<32>) {
  return Array.from(byteArray, function (byte) {
    return ("0" + (byte & 0xff).toString(16)).slice(-2);
  }).join("");
}

export function parseBlock(data: any) {
  let index: number = data.index;
  let timestamp: number = data.timestamp;
  let hash: string = toHexString(data.hash);
  let proofOfWork: number = data.difficulty;
  let nonce: number = data.nonce;

  let transactions: Transaction[] = data.transactions.map(
    (txn: any, _idx: any) => {
      let transactionInputs: TransactionInput[] = txn.inputs.map(
        (i: any, _idx: any) => {
          let ip: TransactionInput = {
            id: toHexString(i.id),
            out: i.out,
            signature: toHexString(i.signature),
            publicKey: toHexString(i.public_key),
          };

          return ip;
        }
      );

      let transactionOutputs: TransactionOutput[] = txn.outputs.map(
        (i: any, _idx: any) => {
          let op: TransactionOutput = {
            value: i.value,
            publicKeyHash: toHexString(i.public_key_hash),
          };

          return op;
        }
      );

      let t: Transaction = {
        id: txn.id,
        inputs: transactionInputs,
        outputs: transactionOutputs,
      };

      return t;
    }
  );

  let b: Block = {
    index,
    timestamp,
    hash,
    proofOfWork,
    nonce,
    transactions,
  };

  return b;
}

export function parseBlockchain(data: any) {
  let blockchain: Blockchain = {
    length: data.length,
    blocks: data.blocks.map((block: any, _i: number) => parseBlock(block)),
  };

  return blockchain;
}
