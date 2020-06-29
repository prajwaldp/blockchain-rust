import React, { useState, useEffect, useRef } from "react";
import "./App.css";

function App() {
  const ws = useRef<WebSocket | null>(null);
  const [messages, setMessages] = useState<Array<any>>([]);
  const [message, setMessage] = useState<string>("");

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
      setMessages([...messages, JSON.parse(message)]);
    } catch (e) {
      console.log(message);
      console.log(e);
    }
  }, [message]);

  return <div className="App">{JSON.stringify(messages)}</div>;
}

export default App;
