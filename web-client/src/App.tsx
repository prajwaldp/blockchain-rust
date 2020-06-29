import React, { useState, useEffect, useRef } from "react";
import "./App.css";

function App() {
  const ws = useRef<WebSocket | null>(null);
  const [messages, setMessages] = useState<Array<any>>([]);

  useEffect(() => {
    ws.current = new WebSocket("ws://127.0.0.1:3012");

    ws.current.onopen = (ev: Event) => {
      console.log("WebSocket connection opened");
    };

    ws.current.onclose = (ev: Event) => {
      console.log("WebSocket connection closed");
    };

    ws.current.onmessage = (e: MessageEvent) => {
      console.log(e.data);
      setMessages([...messages, JSON.parse(e.data)]);
    };

    return () => {
      ws.current?.close();
    };
  }, []);

  return <div className="App">{JSON.stringify(messages)}</div>;
}

export default App;
