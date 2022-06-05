# SmartFactory-revisited
[![Rust](https://github.com/ankokovin/smartFactory-Rust/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/ankokovin/smartFactory-Rust/actions/workflows/rust.yml)

This project is an attempt to [rewrite my program for master's finals](https://github.com/ankokovin/smartFactory-miltiagentagentSim).

## Backend
The initial backend was written in TypeScript with the intent to compiling to JavaScript. This allowed to "merge" the Client and the Backend to one page application, which is hosted with Github Pages and is available at https://ankokovin.github.io/smartFactory-miltiagentagentSim/.

However TypeScript-to-JavaScript compilation is not the only way to "reuse" code in such way. Probably much better experience would be achieved if the target was WASM, not JS.

Also, one of the motivations for the previous project was a prospect of "reusing" code defining agent's behavior in "real agents" - other programs working on real production hardware. However, such claims are kind of dubious... Who would want to use TypeScript in embeded systems?

And the final motivation for this project is educational value. I got an itch to get a bit more into Rust.

TODO:
- [x] Implement a config for WASM compilation
- [ ] Implement async socket server (created a sync for one client for now). Following https://youtu.be/Iapc-qGTEBQ?t=1649 
- [ ] Port the environment class from previous TS project
- [ ] Some more features inside the smart factory model could be nice.

## Frontend
The initial frontend is a single page based on vanilla JS. The reason for not using a framework such as React was that:
1. I'm not really a frontend dev
2. I was not sure how to import the "compiled backend"

If my passion does not run out too fast, I plan to redo frontend as a React app.

TODO:
- [x] Default React App
- [x] Import wasm module
- [ ] Implement socket connection to server
- [ ] Dynamic wasm import?
- [ ] Implement an interface for agent modeling (awaiting backend)  
- [ ] More and better stats
- [ ] Settings storage (could also be a backend? idk)
