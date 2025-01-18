# 🎰 sparse-slot

A lightning-fast, memory-efficient sparse slot map implementation in Rust.

## ✨ Features

- 🚀 **Fixed-size Power**: Pre-allocated capacity for predictable performance
- 🎯 **Safe Access**: Generation-based handles prevent the "dangling pointer blues"
- 🔄 **Reusable Slots**: Removed items' slots can be reused, like a game of musical chairs
- 🎭 **Double Life**: Values can be accessed both immutably and mutably

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sparse-slot = "0.0.3"
```

## 🛠️ Usage

Here's a quick example to get you started:

```rust
use sparse_slot::SparseSlot;

fn main() {
    let mut slot = SparseSlot::new(5);
    let id = slot.try_set("Hello, world!").expect("failed to set");
    println!("Stored value: {:?}", slot.get(id));
}
```

## About Contributions

This project is open source with a single copyright holder (that's me!). While the code is publicly available under the [MIT License](LICENSE), I'm not accepting external contributions at this time.

If you have suggestions or stumble upon bugs, please open an issue for discussion. While I can't accept pull requests, your feedback is invaluable and helps make the project better.

Thank you for your understanding and interest in this project! Your engagement means the world to me. 🙏

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

_Copyright (c) 2024 [Peter Bjorklund](https://github.com/piot). All rights reserved._
