import React, { useState, useEffect, useRef } from "react";
import "./App.css";
import { isPrimitive } from "util";

interface Node {
  id: string; // node name
  messages: string[];
  neighbourCount: number;
  blockchain: Blockchain;
}

interface Wallet {
  id: string; // wallet address
  balance: number;
}

interface TransactionInput {
  id: string;
  out: number;
  signature: string;
  publicKey: string;
}

interface TransactionOutput {
  value: number;
  publicKeyHash: string;
}

interface Transaction {
  id: string;
  inputs: TransactionInput[];
  outputs: TransactionOutput[];
}

interface Block {
  index: number;
  timestamp: number;
  hash: string;
  proofOfWork: number;
  nonce: number;
  transactions: Transaction[];
}

interface Blockchain {
  length: number;
  blocks: Block[];
}

function toHexString(byteArray: Array<32>) {
  return Array.from(byteArray, function (byte) {
    return ("0" + (byte & 0xff).toString(16)).slice(-2);
  }).join("");
}

function processRawBlockData(data: any) {
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

function processRawBlockchainData(data: any) {
  let blockchain: Blockchain = {
    length: data.length,
    blocks: data.blocks.map((block: any, _i: number) =>
      processRawBlockData(block)
    ),
  };

  return blockchain;
}

function App() {
  const ws = useRef<WebSocket | null>(null);
  const [messages, setMessages] = useState<Array<any>>([]);
  const [message, setMessage] = useState<string>("{}");

  const [nodes, setNodes] = useState<Array<Node>>([]);
  const [wallets, setWallets] = useState<Array<Wallet>>([]);

  useEffect(() => {
    ws.current = new WebSocket("ws://127.0.0.1:3012");

    ws.current.onopen = (ev: Event) => {
      console.log("WebSocket connection opened");
    };

    ws.current.onclose = (ev: Event) => {
      console.log("WebSocket connection closed");
    };

    ws.current.onmessage = (e: MessageEvent) => {
      setMessage(e.data);
    };

    return () => {
      ws.current?.close();
    };
  }, []);

  useEffect(() => {
    try {
      let j = JSON.parse(message);
      setMessages([...messages, j]);

      if (j.nodeId === "Main" && j.eventId === "SpawnedNode") {
        let newNode: Node = {
          id: j.details.nodeId,
          messages: [],
          neighbourCount: 0,
          blockchain: { length: 0, blocks: [] },
        };
        setNodes([...nodes, newNode]);
      } else if (j.nodeId === "Main" && j.eventId === "CreatedWallet") {
        let newWallet: Wallet = { id: j.details.walletAddress, balance: 0 };
        setWallets([...wallets, newWallet]);
      } else if (j.eventId === "UpdatedRoutingInfo") {
        let updatedNodes = nodes.map((node, _idx) => {
          if (node.id === j.nodeId) {
            return {
              ...node,
              neighbourCount: j.details.neighbourCount,
            };
          }

          return node;
        });

        setNodes(updatedNodes);
      } else if (j.eventId === "CreatedBlockchain") {
        let blockchain: Blockchain = processRawBlockchainData(
          j.details.rawBlockchainData
        );

        let updatedNodes = nodes.map((node, _idx) => {
          if (node.id === j.nodeId) {
            return {
              ...node,
              blockchain,
            };
          }

          return node;
        });

        setNodes(updatedNodes);
      } else if (j.eventId === "ReceivedFresherBlockchain") {
        let blockchain: Blockchain = processRawBlockchainData(
          j.details.rawBlockchainData
        );

        console.log(j.details.rawBlockchainData);

        let updatedNodes = nodes.map((node, _idx) => {
          if (node.id === j.nodeId) {
            return {
              ...node,
              blockchain,
            };
          }

          return node;
        });

        setNodes(updatedNodes);
      } else if (j.eventId === "ReceivedNewBlock") {
        let updatedNodes = nodes.map((node, _idx) => {
          if (node.id === j.nodeId) {
            let updatedMessages = [
              ...node.messages,
              "Received a new candidate block",
            ];
            return {
              ...node,
              messages: updatedMessages,
            };
          }

          return node;
        });

        setNodes(updatedNodes);
      } else {
        console.log("Unprocessed: ", j);
      }
    } catch (e) {
      console.log(e, message);
    }
  }, [message]);

  return (
    <div className="App">
      <h1>Nodes</h1>
      {nodes.map((node, i) => {
        return (
          <div key={i}>
            {node.id}: {JSON.stringify(node.messages)} : {node.neighbourCount}
            <p>Blockchain ({node.blockchain.length} blocks)</p>
            {node.blockchain.blocks.map((block, i) => {
              return (
                <div key={i}>
                  <p>index: {block.index}</p>
                  <p>timestamp: {block.timestamp}</p>
                  <p>hash: {block.hash}</p>
                  <p>proof of work: {block.proofOfWork}</p>
                  <p>nonce: {block.nonce}</p>
                  <p>Transactions: {JSON.stringify(block.transactions)}</p>
                </div>
              );
            })}
          </div>
        );
      })}

      <h1>Wallets</h1>
      <ul>
        {wallets.map((wallet, i) => {
          return <li key={i}>{wallet.id}</li>;
        })}
      </ul>
    </div>
  );
}

export default App;
