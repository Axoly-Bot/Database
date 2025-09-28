use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
    pub tree: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TreeOperation {
    pub tree: String,
    pub key: String,
    pub value: Option<String>,
}

pub struct SledClient {
    client: Client,
    base_url: String,
}

impl SledClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    // 🔥 OPERACIONES CON TREES (ÁRBOLES)

    /// Insertar un valor en un árbol específico
    pub async fn tree_insert(&self, tree: &str, key: &str, value: &str) -> Result<String> {
        let operation = TreeOperation {
            tree: tree.to_string(),
            key: key.to_string(),
            value: Some(value.to_string()),
        };

        let url = format!("{}/tree/insert", self.base_url);
        let response = self.client
            .post(&url)
            .json(&operation)
            .send()
            .await?;

        Self::handle_response(response).await
    }

    /// Obtener un valor de un árbol específico
    pub async fn tree_get(&self, tree: &str, key: &str) -> Result<Option<String>> {
        let url = format!("{}/tree/get/{}/{}", self.base_url, tree, key);
        let response = self.client
            .get(&url)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                // El servidor devuelve el valor como JSON string, necesitamos deserializarlo
                let json_string = response.text().await?;
                // Remover las comillas JSON si existen
                let value = Self::strip_json_quotes(&json_string);
                Ok(Some(value))
            }
            StatusCode::NOT_FOUND => Ok(None),
            _ => {
                let error_msg = response.text().await?;
                Err(anyhow!("Error: {}", error_msg))
            }
        }
    }

    /// Eliminar un valor de un árbol específico
    pub async fn tree_delete(&self, tree: &str, key: &str) -> Result<String> {
        let url = format!("{}/tree/delete/{}/{}", self.base_url, tree, key);
        let response = self.client
            .delete(&url)
            .send()
            .await?;

        Self::handle_response(response).await
    }

    /// Listar todas las claves de un árbol específico
    pub async fn tree_list_keys(&self, tree: &str) -> Result<Vec<String>> {
        let url = format!("{}/tree/list/{}", self.base_url, tree);
        let response = self.client
            .get(&url)
            .send()
            .await?;

        let json_string = response.text().await?;
        // Deserializar el JSON array
        let keys: Vec<String> = serde_json::from_str(&json_string)?;
        Ok(keys)
    }

    /// Listar todos los árboles disponibles
    pub async fn list_all_trees(&self) -> Result<Vec<String>> {
        let url = format!("{}/trees", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await?;

        let json_string = response.text().await?;
        // Deserializar el JSON array
        let trees: Vec<String> = serde_json::from_str(&json_string)?;
        Ok(trees)
    }

    // 📋 OPERACIONES LEGACY (árbol principal)

    /// Insertar en el árbol principal (legacy)
    pub async fn insert(&self, key: &str, value: &str) -> Result<String> {
        let kv = KeyValue {
            key: key.to_string(),
            value: value.to_string(),
            tree: None,
        };

        let url = format!("{}/insert", self.base_url);
        let response = self.client
            .post(&url)
            .json(&kv)
            .send()
            .await?;

        Self::handle_response(response).await
    }

    /// Obtener del árbol principal (legacy)
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let url = format!("{}/get/{}", self.base_url, key);
        let response = self.client
            .get(&url)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                // El servidor devuelve el valor como JSON string
                let json_string = response.text().await?;
                // Remover las comillas JSON si existen
                let value = Self::strip_json_quotes(&json_string);
                Ok(Some(value))
            }
            StatusCode::NOT_FOUND => Ok(None),
            _ => {
                let error_msg = response.text().await?;
                Err(anyhow!("Error: {}", error_msg))
            }
        }
    }

    /// Verificar salud del servidor
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await?;

        if response.status().is_success() {
            let text = response.text().await?;
            Ok(text.trim_matches('"') == "OK")
        } else {
            Ok(false)
        }
    }

    // Método auxiliar para manejar respuestas
    async fn handle_response(mut response: Response) -> Result<String> {
        let status = response.status();
        if status.is_success() {
            let text = response.text().await?;
            // Remover comillas JSON de la respuesta
            Ok(Self::strip_json_quotes(&text))
        } else {
            let error_msg = response.text().await?;
            Err(anyhow!("HTTP {}: {}", status, error_msg))
        }
    }

    // Función auxiliar para remover comillas JSON
    fn strip_json_quotes(json_string: &str) -> String {
        json_string.trim_matches('"').to_string()
    }
}