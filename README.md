xtea is an empty application with little features.

It is meant as a container of plugins. Without plugins, it's just an empty ImGui window.

# Installing

To complile, you have to have rust. Install rust at [](https://www.rust-lang.org/tools/install).

Once you have rust, compile xtea with `cargo build --release`

# Install plugins

To install a plugin, just place the `.dll` (windows) or `.so` (linux) inside a folder named `plugins`.

# Develop plugin

You can see plugin examples in the [examples](./examples/) directory.
For more information check the [plugin guide](./plugin%20guide.md)
