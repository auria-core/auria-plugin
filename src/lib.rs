use auria_core::AuriaResult;
use async_trait::async_trait;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn initialize(&self) -> AuriaResult<()>;
    async fn shutdown(&self) -> AuriaResult<()>;
}

pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub async fn initialize_all(&self) -> AuriaResult<()> {
        for plugin in &self.plugins {
            plugin.initialize().await?;
        }
        Ok(())
    }

    pub async fn shutdown_all(&self) -> AuriaResult<()> {
        for plugin in &self.plugins {
            plugin.shutdown().await?;
        }
        Ok(())
    }
}
