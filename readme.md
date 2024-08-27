# Rugui

## Description

`rugui` is a Rust library for building graphical user interfaces (GUIs) with a focus on real-time frame updates and high-performance rendering. Designed and optimized for applications requiring smooth and responsive interfaces, such as games and interactive simulations, `rugui` leverages modern GPU-accelerated rendering through the `wgpu` library.

By providing a flexible and efficient framework for creating dynamic and interactive UIs, `rugui` enables developers to build visually rich and high-performance applications with minimal overhead. Its declarative approach to UI development, coupled with support for advanced styling and responsive layouts, makes it ideal for applications where real-time updates and fluid interactions are critical.

## Features

- **GPU-Accelerated Rendering:** Utilizes `wgpu` for efficient, hardware-accelerated rendering, providing smooth and high-performance graphics.
- **Declarative UI:** Supports a declarative approach to UI development, allowing developers to describe layouts and styles in a straightforward and intuitive manner.
- **Flexible Styling:** Offers extensive styling options, including colors, gradients, textures, and transformations, to create visually appealing and customizable interfaces.
- **Responsive Layouts:** Provides support for flexible and responsive layouts, including rows, columns, and custom arrangements, to adapt to different screen sizes and resolutions.
- **Event Handling:** Includes a robust event handling system for user interactions, such as mouse movements and clicks, allowing for interactive and dynamic UIs.
- **Element Management:** Allows easy addition, modification, and removal of UI elements, making it simple to build and update complex user interfaces.
- **Dynamic Updates:** Supports real-time updates to the UI, ensuring that changes are reflected immediately and efficiently.
- **Integration with `winit`:** Seamlessly integrates with the `winit` library for window management and event handling, streamlining the process of setting up and managing application windows.

With `rugui`, you can build sophisticated and performant GUIs for your Rust applications, taking advantage of modern GPU capabilities while keeping your codebase clean and maintainable.

## Getting Started

Creating a `rugui` application involves several key steps, from initializing the GUI to adding and styling elements, handling events, and properly displaying frames. This guide will walk you through these steps, assuming you have already set up the necessary `wgpu` resources.

### 1. Initialize the Rugui GUI

Before you can initialize the `Gui` object, you'll need to have your `wgpu` setup ready, including a `device`, `queue`, and surface configuration. If you haven't set this up yet, you can refer to the [wgpu guide](https://sotrh.github.io/learn-wgpu/) for detailed instructions.

Once your `wgpu` setup is ready, you can initialize the `Gui` object as follows:

```rust
use rugui::Gui;

// Assume `device` and `queue` are already created, and you have the window size available
let mut gui: Gui<()> = Gui::new((window_width, window_height), &device, &queue);
```

### 2. Add and Style Elements

After initializing the `Gui`, you can add UI elements such as rows and columns and apply styles to them. Here's an example:

```rust
use rugui::{Element, Color, LinearGradient, ColorPoint, Position, Size};

let mut row1 = Element::new().with_label("Row 1");
row1.text_str("Hello, world!");
row1.styles.text_mut().color = Color::GREEN;

let mut column1 = Element::new().with_label("Column 1");
column1.styles.set_bg_lin_gradient(Some(LinearGradient {
    p1: ColorPoint {
        position: Position::Top,
        color: Color::RED.with_alpha(0.3),
    },
    p2: ColorPoint {
        position: Position::Center,
        color: Color::TRANSPARENT,
    },
}));

// Add more elements as needed...
```

### 3. Set an Entry Point

To display your UI, set an entry point for the Gui. This entry point defines the root of your GUI layout:`

```rust
let entry = gui.add_element(row1);
gui.set_entry(Some(entry));
```

### 4. Handle Resize Events

Handling window resizing is important for creating a responsive application. Here’s how you can handle resizing:

```rust
fn handle_resize(gui: &mut Gui<()>, new_size: (u32, u32)) {
    if new_size.0 > 0 && new_size.1 > 0 {
        gui.resize(new_size, &queue);
    }
}

// Use this in your event loop
```

### 5. Properly Display a Frame

To render the GUI correctly, you must update the GUI state, prepare it for rendering, and then draw it:

```rust
fn render_frame(gui: &mut Gui<()>) {
    // Update the GUI state
    gui.update();

    // Prepare the GUI for rendering
    gui.prepare(&device, &queue);

    // Assume the necessary `wgpu` resources like `output`, `view`, and `encoder` are set up
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
    });

    gui.render(&mut pass);
    queue.submit(std::iter::once(encoder.finish()));
    output.present();
}
```

### Running Examples

To get a hands-on understanding of how `rugui` works, you can explore the example files included in the repository. Two particularly useful examples are `hello.rs` and `full.rs`.

- `hello.rs`: A simple example that demonstrates the basic setup and a minimal UI.
- `full.rs`: A more comprehensive example that showcases the full capabilities of rugui, including complex layouts and event handling.

To run these examples, it's recommended to run them in release mode, as some of the calculations may not perform optimally in debug mode. Use the following command:

```bash
cargo run --release --example hello
```

Some examples may require additional features. For example, the `full.rs` example uses the `winit` feature, which can be enabled as follows:

```bash
cargo run --release --example full --features winit
```

Running in release mode ensures smoother performance, especially for more complex examples.

## Troubleshooting

### 1. GUI Not Rendering Properly

- Ensure that your `wgpu` setup is correct and the device, queue, and surface configuration are properly initialized.
- Check if the `Gui` object has been correctly prepared and updated before rendering.

### 2. Resize Events Not Handled

- Verify that you are calling the `handle_resize` function correctly in your event loop and that the `gui.resize` method is being invoked with the updated window size.

### 3. Performance Issues

- Make sure you are running examples in release mode for optimal performance. Use `cargo run --release` to avoid debug mode overhead.
- Check for any unnecessary computations or excessive updates in your event handling and rendering code.

For more detailed troubleshooting, please refer to the [rugui GitHub Issues page](https://github.com/it-2001/rugui/issues) or the [discussion forum](https://github.com/it-2001/rugui/issues).
