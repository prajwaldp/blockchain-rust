import React, { useState, useEffect, useRef } from "react";
import "./App.css";
import "./assets/tailwind.generated.css";

import { Node, Wallet, Blockchain } from "./models";
import { parseBlockchain } from "./util";

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
        let blockchain: Blockchain = parseBlockchain(
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
        let blockchain: Blockchain = parseBlockchain(
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
      <header className="p-4 text-4xl font-light text-center text-blue-600">
        <h1>Real-time Blockchain Dashboard</h1>
      </header>
      <div className="flex p-5 bg-gray-100">
        <div className="w-1/2">
          <h1 className="text-2xl text-gray-700 font-bold text-center">
            Nodes in the Network
          </h1>
          <p className="text-sm text-gray-500 text-center">
            {nodes.length} nodes in the network
          </p>

          <div className="p-5">
            {nodes.map((node, i) => {
              return (
                <div key={i} className="bg-white p-3 rounded mb-5">
                  <h3 className="text-2xl text-blue-300 mb-3">{node.id}</h3>

                  <p className="text-sm text-gray-600">
                    Connected to {node.neighbourCount}, Blockchain copy has{" "}
                    {node.blockchain.length} block(s), And has{" "}
                    {node.messages.length} message(s)
                  </p>

                  <p className="text-xl text-gray-600 mt-3">Blocks:</p>

                  {node.blockchain.blocks.map((block, i) => {
                    return (
                      <div
                        key={i}
                        className="bg-gray-200 text-sm p-3 mb-2 text-gray-600"
                      >
                        <p className="font-bold">Index: {block.index}</p>
                        <p>Timestamp: {block.timestamp}</p>
                        <p>Hash: {block.hash}</p>
                        <p>Proof of Work: {block.proofOfWork}</p>
                        <p>Nonce: {block.nonce}</p>
                        {block.transactions.map((t, idx) => {
                          return (
                            <div className="border-gray-300 p-2" key={idx}>
                              <p className="truncate">Transaction ID: {t.id}</p>

                              <p>Inputs</p>
                              {t.inputs.map((ip, ip_idx) => {
                                return (
                                  <div
                                    className="bg-gray-900 p-1 text-gray-200"
                                    key={ip_idx}
                                  >
                                    <p>ID: {ip.id}</p>
                                    <p>Out: {ip.out}</p>
                                    <p>Signature: {ip.signature}</p>
                                    <p>Public Key: {ip.publicKey}</p>
                                  </div>
                                );
                              })}

                              <p>Outputs</p>
                              {t.outputs.map((op, op_idx) => {
                                return (
                                  <div
                                    className="bg-gray-900 p-1 text-gray-200"
                                    key={op_idx}
                                  >
                                    <p>Value: {op.value}</p>
                                    <p>Public Key: {op.publicKeyHash}</p>
                                  </div>
                                );
                              })}
                            </div>
                          );
                        })}
                      </div>
                    );
                  })}
                </div>
              );
            })}
          </div>
        </div>

        <div className="w-1/2 border-l-2">
          <h1 className="text-2xl text-gray-700 font-bold text-center">
            Wallets
          </h1>
          <p className="text-sm text-gray-500 text-center">
            {wallets.length} wallets registered
          </p>
          {wallets.map((wallet, i) => {
            return (
              <div className="text-sm text-gray-600" key={i}>
                {wallet.id}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

export default App;
