# SmartFactory-revisited
[![Rust](https://github.com/ankokovin/smartFactory-Rust/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/ankokovin/smartFactory-Rust/actions/workflows/rust.yml)

This project is an attempt to [rewrite my program for master's finals](https://github.com/ankokovin/smartFactory-miltiagentagentSim).

## Backend
The initial backend was written in TypeScript with the intent to compiling to JavaScript. This allowed to "merge" the Client and the Backend to one html page (which was hosted with GitHub Pages and is probably still available at https://ankokovin.github.io/smartFactory-miltiagentagentSim/).

However, TypeScript-to-JavaScript compilation is not the only way to "reuse" code in such way. Probably much better experience would be achieved if the target was WASM, not JS.

Also, one of the motivations for the previous project was a prospect of "reusing" code defining agent's behavior in "real agents" - other programs working on real production hardware. However, such claims are kind of dubious... Who would want to use TypeScript in embedded systems?

And the final motivation for this project is educational value. I got an itch to get a bit more into Rust.

TODO:
- [x] Implement a config for WASM compilation
- [x] Implement async socket server
- [ ] Port the environment class from previous TS project
- [ ] Some more features inside the smart factory model could be nice

## Frontend
The initial frontend is a single page based on vanilla JS. The reason for not using a framework such as React was that:
1. I'm not really a frontend dev
2. I was not sure how to import the "compiled backend"

If my passion does not run out too fast, I plan to redo frontend as a React app.

TODO:
- [x] Default React App
- [x] Import wasm module
- [x] Implement socket connection to server
- [ ] Dynamic wasm import?
- [ ] Implement an interface for agent modeling (awaiting backend)  
- [ ] More and better stats
- [ ] Settings storage (could also be a backend? idk)


## Future vision
Actually, it doesn't really feel satisfying to just recreate my course work in Rust + WASM + React. I think it is possible to dig a bit deeper. 
For example, one of "notes" from the people grading my previous work was that it could've been much more general, more applicable to other domains.

It would be nice to rectify that.

My plan is to expand on usage of WASM to achieve a "plugin-able" system, where plugins are WASM modules. Of course, at the time of writing this line (June 7th 2022) the (Component Model Proposal)[https://github.com/WebAssembly/component-model?ref=https://githubhelp.com] is still in phase 1. I don't really know how much will be possible, but some systems have succeeded in using WASM this way. This means that under the hood of both frontend and backend (Rust server) such mechanism would be could be implemented.

One problem is that this sounds a bit out of scope in context of available time period, so only time will tell if I will have motivation to implement this :^)

TODO:
- [ ] Implement a simple agent model (no plugin)
- [ ] Reimplement my smart factory model
- [ ] Implement a WASM-based plugin system on frontend, compile both models for testing
- [ ] Implement a WASM-based plugin system on Rust server, test both models
- [ ] Rename the repository to reflect its new status
- [ ] Maybe implement some tooling for agent model creation? I kinda want to revisit project about (creating a visual programming environment)[https://github.com/ankokovin/BlocklyForHouse] if only for the proof of concept.