
This project can generate a web UI based on a [Clap](https://docs.rs/clap/latest/clap/) command.

It maps the cli args to HTML input elements. When click a button, it passes in the inputs as a Clap structure and call user defined function (the function need to be compiled to WASM).
