// File: lib.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Plugin system for extending AURIA Runtime Core.
//     Provides dynamic loading and management of plugins for extensibility,
//     allowing custom backends, routing algorithms, and hardware support.
//
use auria_core::{AuriaResult, Tier, Tensor, ExecutionState, ExecutionOutput};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn initialize(&self) -> AuriaResult<()>;
    async fn shutdown(&self) -> AuriaResult<()>;
}

pub struct PluginRegistry {
    plugins: Arc<RwLock<Vec<Box<dyn Plugin>>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn register(&self, plugin: Box<dyn Plugin>) {
        self.plugins.write().await.push(plugin);
    }

    pub async fn initialize_all(&self) -> AuriaResult<()> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.iter() {
            plugin.initialize().await?;
        }
        Ok(())
    }

    pub async fn shutdown_all(&self) -> AuriaResult<()> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.iter() {
            plugin.shutdown().await?;
        }
        Ok(())
    }

    pub async fn list_plugins(&self) -> Vec<PluginInfo> {
        let plugins = self.plugins.read().await;
        plugins.iter()
            .map(|p| PluginInfo {
                name: p.name().to_string(),
                version: p.version().to_string(),
            })
            .collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
}

pub struct PluginManager {
    registry: Arc<PluginRegistry>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(PluginRegistry::new()),
        }
    }

    pub fn registry(&self) -> Arc<PluginRegistry> {
        self.registry.clone()
    }

    pub async fn load_plugin(&self, plugin: Box<dyn Plugin>) -> AuriaResult<()> {
        self.registry.register(plugin).await;
        Ok(())
    }

    pub async fn unload_plugin(&self, _name: &str) -> AuriaResult<()> {
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_registry() {
        let registry = PluginRegistry::new();
        
        struct TestPlugin;
        
        #[async_trait]
        impl Plugin for TestPlugin {
            fn name(&self) -> &str { "test" }
            fn version(&self) -> &str { "1.0.0" }
            async fn initialize(&self) -> AuriaResult<()> { Ok(()) }
            async fn shutdown(&self) -> AuriaResult<()> { Ok(()) }
        }

        registry.register(Box::new(TestPlugin)).await;
        registry.initialize_all().await.unwrap();
    }
}
