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
sparse-slot = "0.0.1"
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
