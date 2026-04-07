# auria-plugin

Plugin system for extending AURIA Runtime Core.

## Overview

Provides dynamic loading and management of plugins for extensibility.

## Plugin Trait

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn initialize(&self) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
}
```

## Usage

```rust
use auria_plugin::{PluginRegistry, MyPlugin};

let mut registry = PluginRegistry::new();
registry.register(Box::new(MyPlugin::new()));
registry.initialize_all().await?;
```
