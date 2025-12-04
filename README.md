
This project can generate a web UI based on a [Clap](https://docs.rs/clap/latest/clap/) command.

It maps the cli args to HTML input elements. When click a button, it passes in the inputs as a Clap structure and call user defined function (the function need to be compiled to WASM).

## Example

There is an example under `example/` project.

Build WASM files with this command first:

```
~/.cargo/bin/wasm-pack build --target web
```

It will create files under `pkg`.

Then run

```
cargo run --bin generate_ui
```

It will generate a `generated_ui.html` file that you can open in the browser that can run the exported `process_bind` with web UI.
