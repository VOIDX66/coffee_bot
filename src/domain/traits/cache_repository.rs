// cache_repository.rs
use async_trait::async_trait;
use anyhow::Result;

/* Cache Repository
Este trait define la interfaz para el repositorio de caché, permitiendo: 
- Obtener un valor por su clave
- Guardar un valor con una clave y un tiempo de expiración
Esto sin importar el tipo de valor almacenado
*/

#[async_trait]
pub trait CacheRepository<T>: Send + Sync {
  async fn get(&self, key: &str) -> Result<Option<T>>;
  async fn set(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<()>;
}