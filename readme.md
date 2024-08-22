# Rugui

Rugui is a simple GUI library for projects using Rust and WGPU. It is designed to be easy to use and to provide a simple way to create interactive applications.

The main intention of this library is to have a flexible and easy to use GUI library that can be used in any project that uses WGPU. Mainly for applications with continuous rendering, such as games, simulations, etc.

## Features

- Easy to use: The library is designed to be easy to use and to have a simple API.
- Persistent elements: Elements are not recreated every frame, they are only created once and updated every frame if necessary.
- Customizable: The library is designed to be customizable, you can create your own elements and customize the existing ones.
- No dependencies: The library has no dependencies by default, but you can use feature flags to enable some dependencies if you want.


### Flags

- `winit`: Enables integration with winit events.
- `serde`: Enables serialization and deserialization of the GUI state.
- `image`: Provides tools for working with images.
- `elements`: Provides a collection of common elements.

## Examples

For examples, see the `examples` directory.