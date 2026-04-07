// File: lib.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Plugin system for extending AURIA Runtime Core.
//     Provides dynamic loading and management of plugins for extensibility,
//     allowing custom backends, routing algorithms, and hardware support.
//
use auria_core::{AuriaError, AuriaResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn plugin_type(&self) -> PluginType;
    async fn initialize(&self) -> AuriaResult<()>;
    async fn shutdown(&self) -> AuriaResult<()>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    Backend,
    Router,
    Middleware,
    Storage,
    Security,
    Monitoring,
    Custom(String),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PluginHooks {
    pub pre_execution: bool,
    pub post_execution: bool,
    pub pre_routing: bool,
    pub post_routing: bool,
    pub on_error: bool,
    pub on_request: bool,
    pub on_response: bool,
}

impl PluginHooks {
    pub fn all() -> Self {
        Self {
            pre_execution: true,
            post_execution: true,
            pre_routing: true,
            post_routing: true,
            on_error: true,
            on_request: true,
            on_response: true,
        }
    }

    pub fn none() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub plugin_type: PluginType,
    pub description: String,
    pub author: String,
    pub dependencies: Vec<String>,
    pub hooks: PluginHooks,
    pub enabled: bool,
    pub loaded_at: u64,
}

impl PluginMetadata {
    pub fn new(name: String, version: String, plugin_type: PluginType) -> Self {
        Self {
            name,
            version,
            plugin_type,
            description: String::new(),
            author: String::new(),
            dependencies: Vec::new(),
            hooks: PluginHooks::none(),
            enabled: true,
            loaded_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<String, PluginEntry>>>,
}

struct PluginEntry {
    metadata: PluginMetadata,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register<P: Plugin + 'static>(&self, plugin: &P) -> AuriaResult<()> {
        let name = plugin.name().to_string();
        
        if self.plugins.read().await.contains_key(&name) {
            return Err(AuriaError::ExecutionError(
                format!("Plugin {} already registered", name),
            ));
        }
        
        let metadata = PluginMetadata::new(
            name.clone(),
            plugin.version().to_string(),
            plugin.plugin_type(),
        );
        
        self.plugins.write().await.insert(name, PluginEntry { metadata });
        
        Ok(())
    }

    pub async fn unregister(&self, name: &str) -> Option<PluginMetadata> {
        self.plugins.write().await.remove(name).map(|e| e.metadata)
    }

    pub async fn get_metadata(&self, name: &str) -> Option<PluginMetadata> {
        self.plugins.read().await.get(name).map(|e| e.metadata.clone())
    }

    pub async fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins
            .read()
            .await
            .values()
            .map(|e| PluginInfo {
                name: e.metadata.name.clone(),
                version: e.metadata.version.clone(),
                plugin_type: e.metadata.plugin_type.clone(),
                enabled: e.metadata.enabled,
            })
            .collect()
    }

    pub async fn list_by_type(&self, plugin_type: PluginType) -> Vec<PluginInfo> {
        self.plugins
            .read()
            .await
            .values()
            .filter(|e| e.metadata.plugin_type == plugin_type)
            .map(|e| PluginInfo {
                name: e.metadata.name.clone(),
                version: e.metadata.version.clone(),
                plugin_type: e.metadata.plugin_type.clone(),
                enabled: e.metadata.enabled,
            })
            .collect()
    }

    pub async fn enable(&self, name: &str) -> AuriaResult<()> {
        let mut plugins = self.plugins.write().await;
        
        if let Some(entry) = plugins.get_mut(name) {
            entry.metadata.enabled = true;
            Ok(())
        } else {
            Err(AuriaError::ExecutionError(
                format!("Plugin {} not found", name),
            ))
        }
    }

    pub async fn disable(&self, name: &str) -> AuriaResult<()> {
        let mut plugins = self.plugins.write().await;
        
        if let Some(entry) = plugins.get_mut(name) {
            entry.metadata.enabled = false;
            Ok(())
        } else {
            Err(AuriaError::ExecutionError(
                format!("Plugin {} not found", name),
            ))
        }
    }

    pub async fn is_enabled(&self, name: &str) -> bool {
        self.plugins
            .read()
            .await
            .get(name)
            .map(|e| e.metadata.enabled)
            .unwrap_or(false)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub plugin_type: PluginType,
    pub enabled: bool,
}

pub struct PluginManager {
    registry: Arc<PluginRegistry>,
    config: PluginConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugin_dirs: Vec<PathBuf>,
    pub auto_enable: bool,
    pub enable_hot_reload: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            plugin_dirs: Vec::new(),
            auto_enable: true,
            enable_hot_reload: false,
        }
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(PluginRegistry::new()),
            config: PluginConfig::default(),
        }
    }

    pub fn with_config(config: PluginConfig) -> Self {
        Self {
            registry: Arc::new(PluginRegistry::new()),
            config,
        }
    }

    pub fn registry(&self) -> Arc<PluginRegistry> {
        self.registry.clone()
    }

    pub async fn register_plugin<P: Plugin + 'static>(&self, plugin: &P) -> AuriaResult<()> {
        self.registry.register(plugin).await
    }

    pub async fn unregister_plugin(&self, name: &str) -> Option<PluginMetadata> {
        self.registry.unregister(name).await
    }

    pub async fn load_plugins_from_dir(&self, dir: &PathBuf) -> AuriaResult<usize> {
        let mut loaded = 0;
        
        if !dir.exists() {
            return Ok(0);
        }
        
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(0),
        };
        
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.extension().map_or(false, |e| e == "so" || e == "dll" || e == "dylib") {
                loaded += 1;
            }
        }
        
        Ok(loaded)
    }

    pub async fn list_plugins(&self) -> Vec<PluginInfo> {
        self.registry.list_plugins().await
    }

    pub async fn get_plugin_info(&self, name: &str) -> Option<PluginInfo> {
        self.registry.get_metadata(name).await.map(|m| PluginInfo {
            name: m.name,
            version: m.version,
            plugin_type: m.plugin_type,
            enabled: m.enabled,
        })
    }

    pub async fn enable_plugin(&self, name: &str) -> AuriaResult<()> {
        self.registry.enable(name).await
    }

    pub async fn disable_plugin(&self, name: &str) -> AuriaResult<()> {
        self.registry.disable(name).await
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BackendPlugin;

#[async_trait]
impl Plugin for BackendPlugin {
    fn name(&self) -> &str { "custom-backend" }
    fn version(&self) -> &str { "1.0.0" }
    fn plugin_type(&self) -> PluginType { PluginType::Backend }
    
    async fn initialize(&self) -> AuriaResult<()> { Ok(()) }
    async fn shutdown(&self) -> AuriaResult<()> { Ok(()) }
}

pub struct RouterPlugin;

#[async_trait]
impl Plugin for RouterPlugin {
    fn name(&self) -> &str { "custom-router" }
    fn version(&self) -> &str { "1.0.0" }
    fn plugin_type(&self) -> PluginType { PluginType::Router }
    
    async fn initialize(&self) -> AuriaResult<()> { Ok(()) }
    async fn shutdown(&self) -> AuriaResult<()> { Ok(()) }
}

pub struct MiddlewarePlugin;

#[async_trait]
impl Plugin for MiddlewarePlugin {
    fn name(&self) -> &str { "custom-middleware" }
    fn version(&self) -> &str { "1.0.0" }
    fn plugin_type(&self) -> PluginType { PluginType::Middleware }
    
    async fn initialize(&self) -> AuriaResult<()> { Ok(()) }
    async fn shutdown(&self) -> AuriaResult<()> { Ok(()) }
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
            fn plugin_type(&self) -> PluginType { PluginType::Custom("test".to_string()) }
            async fn initialize(&self) -> AuriaResult<()> { Ok(()) }
            async fn shutdown(&self) -> AuriaResult<()> { Ok(()) }
        }

        let plugin = TestPlugin;
        registry.register(&plugin).await.unwrap();
        
        let plugins = registry.list_plugins().await;
        assert_eq!(plugins.len(), 1);
    }

    #[tokio::test]
    async fn test_plugin_enable_disable() {
        let registry = PluginRegistry::new();
        
        struct TestPlugin;
        
        #[async_trait]
        impl Plugin for TestPlugin {
            fn name(&self) -> &str { "test" }
            fn version(&self) -> &str { "1.0.0" }
            fn plugin_type(&self) -> PluginType { PluginType::Custom("test".to_string()) }
            async fn initialize(&self) -> AuriaResult<()> { Ok(()) }
            async fn shutdown(&self) -> AuriaResult<()> { Ok(()) }
        }

        let plugin = TestPlugin;
        registry.register(&plugin).await.unwrap();
        
        registry.disable("test").await.unwrap();
        assert!(!registry.is_enabled("test").await);
        
        registry.enable("test").await.unwrap();
        assert!(registry.is_enabled("test").await);
    }

    #[tokio::test]
    async fn test_plugin_manager() {
        let manager = PluginManager::new();
        
        struct TestPlugin;
        
        #[async_trait]
        impl Plugin for TestPlugin {
            fn name(&self) -> &str { "test" }
            fn version(&self) -> &str { "1.0.0" }
            fn plugin_type(&self) -> PluginType { PluginType::Custom("test".to_string()) }
            async fn initialize(&self) -> AuriaResult<()> { Ok(()) }
            async fn shutdown(&self) -> AuriaResult<()> { Ok(()) }
        }

        let plugin = TestPlugin;
        manager.register_plugin(&plugin).await.unwrap();
        
        let plugins = manager.list_plugins().await;
        assert_eq!(plugins.len(), 1);
    }

    #[tokio::test]
    async fn test_list_by_type() {
        let registry = PluginRegistry::new();
        
        struct BackendTestPlugin;
        struct RouterTestPlugin;
        
        #[async_trait]
        impl Plugin for BackendTestPlugin {
            fn name(&self) -> &str { "backend-test" }
            fn version(&self) -> &str { "1.0.0" }
            fn plugin_type(&self) -> PluginType { PluginType::Backend }
            async fn initialize(&self) -> AuriaResult<()> { Ok(()) }
            async fn shutdown(&self) -> AuriaResult<()> { Ok(()) }
        }
        
        #[async_trait]
        impl Plugin for RouterTestPlugin {
            fn name(&self) -> &str { "router-test" }
            fn version(&self) -> &str { "1.0.0" }
            fn plugin_type(&self) -> PluginType { PluginType::Router }
            async fn initialize(&self) -> AuriaResult<()> { Ok(()) }
            async fn shutdown(&self) -> AuriaResult<()> { Ok(()) }
        }

        registry.register(&BackendTestPlugin).await.unwrap();
        registry.register(&RouterTestPlugin).await.unwrap();
        
        let backends = registry.list_by_type(PluginType::Backend).await;
        assert_eq!(backends.len(), 1);
    }

    #[tokio::test]
    async fn test_plugin_hooks() {
        let hooks = PluginHooks::all();
        assert!(hooks.pre_execution);
        assert!(hooks.post_execution);
        assert!(hooks.on_error);
        
        let hooks = PluginHooks::none();
        assert!(!hooks.pre_execution);
        assert!(!hooks.post_execution);
    }

    #[tokio::test]
    async fn test_plugin_unregister() {
        let registry = PluginRegistry::new();
        
        struct TestPlugin;
        
        #[async_trait]
        impl Plugin for TestPlugin {
            fn name(&self) -> &str { "test" }
            fn version(&self) -> &str { "1.0.0" }
            fn plugin_type(&self) -> PluginType { PluginType::Custom("test".to_string()) }
            async fn initialize(&self) -> AuriaResult<()> { Ok(()) }
            async fn shutdown(&self) -> AuriaResult<()> { Ok(()) }
        }

        let plugin = TestPlugin;
        registry.register(&plugin).await.unwrap();
        
        let removed = registry.unregister("test").await;
        assert!(removed.is_some());
        
        let plugins = registry.list_plugins().await;
        assert_eq!(plugins.len(), 0);
    }
}
