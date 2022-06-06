import React from 'react';
import logo from './WebAssembly_Logo.svg';
import './App.css';
import asm, {greet} from "@rycarok/smart-factory-wasm-port";

console.log("Hello, world")

let socket = new WebSocket( "ws://localhost:8080");
// @ts-ignore
socket.onopen = () =>socket.send("Hello, World");
socket.onmessage = (message) => console.log(message);
function App() {
  return (
    <div className="App">
      <header className="App-header">
        <button className='spin' onClick={
          () => {
            asm().then(
              () => greet("WebAssembly")
            )
          }
        }>
          <img src={logo} className="App-logo" alt="logo"/>
        </button>
      </header>
    </div>
  );
}

export default App;
