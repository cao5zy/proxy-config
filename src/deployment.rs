//! 镜像版本与部署状态管理。

use crate::config::AppConfig;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeploymentState {
    pub active_image: Option<String>,
    pub previous_image: Option<String>,
    pub candidate_image: Option<String>,
    pub active_container: Option<String>,
    pub previous_container: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeploymentStore {
    path: PathBuf,
    states: HashMap<String, DeploymentState>,
}

pub fn image_reference(app_name: &str, source_hash: &str) -> String {
    format!(
        "{}:sha-{}",
        app_name,
        &source_hash[..source_hash.len().min(12)]
    )
}

pub fn candidate_container_name(container_name: &str, image: &str) -> String {
    let suffix = image
        .rsplit_once("sha-")
        .map(|(_, value)| value)
        .unwrap_or("candidate");
    format!("{}--{}", container_name, suffix)
}

/// 将稳定的应用配置转换为当前活动版本的运行配置。
pub fn active_runtime_apps(
    apps: &[AppConfig],
    states: &HashMap<String, DeploymentState>,
) -> Result<(Vec<AppConfig>, HashMap<String, String>)> {
    let mut runtime_apps = Vec::with_capacity(apps.len());
    let mut images = HashMap::new();
    for app in apps {
        let state = states
            .get(&app.name)
            .ok_or_else(|| Error::State(format!("应用 '{}' 没有活动部署状态", app.name)))?;
        let image = state
            .active_image
            .clone()
            .ok_or_else(|| Error::State(format!("应用 '{}' 没有活动镜像", app.name)))?;
        let container = state
            .active_container
            .clone()
            .ok_or_else(|| Error::State(format!("应用 '{}' 没有活动容器", app.name)))?;
        let mut runtime_app = app.clone();
        runtime_app.container_name = container;
        images.insert(app.name.clone(), image);
        runtime_apps.push(runtime_app);
    }
    Ok((runtime_apps, images))
}

/// 生成一次候选发布所需的运行配置。尚未部署的其他应用保留在配置中，
/// 但不会被赋予不存在的镜像引用。
pub fn runtime_apps_with_candidate(
    apps: &[AppConfig],
    states: &HashMap<String, DeploymentState>,
    candidate_app: &str,
    candidate_image: &str,
) -> Result<(Vec<AppConfig>, HashMap<String, String>, String)> {
    let configured = apps
        .iter()
        .find(|app| app.name == candidate_app)
        .ok_or_else(|| Error::State(format!("未找到应用 '{}'", candidate_app)))?;
    let candidate_container = candidate_container_name(&configured.container_name, candidate_image);
    let mut runtime_apps = Vec::with_capacity(apps.len());
    let mut images = HashMap::new();
    for app in apps {
        let mut runtime = app.clone();
        if app.name == candidate_app {
            runtime.container_name = candidate_container.clone();
            images.insert(app.name.clone(), candidate_image.to_string());
        } else if let Some(state) = states.get(&app.name) {
            if let (Some(image), Some(container)) = (&state.active_image, &state.active_container) {
                runtime.container_name = container.clone();
                images.insert(app.name.clone(), image.clone());
            }
        }
        runtime_apps.push(runtime);
    }
    Ok((runtime_apps, images, candidate_container))
}

impl DeploymentStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            states: HashMap::new(),
        }
    }

    pub fn load(&mut self) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(&self.path)
            .map_err(|e| Error::State(format!("读取部署状态 {:?} 失败: {}", self.path, e)))?;
        self.states = serde_yaml::from_str(&content)
            .map_err(|e| Error::State(format!("解析部署状态 {:?} 失败: {}", self.path, e)))?;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_yaml::to_string(&self.states)
            .map_err(|e| Error::State(format!("序列化部署状态失败: {}", e)))?;
        fs::write(&self.path, content)
            .map_err(|e| Error::State(format!("写入部署状态 {:?} 失败: {}", self.path, e)))
    }

    pub fn state(&self, app_name: &str) -> Option<&DeploymentState> {
        self.states.get(app_name)
    }

    pub fn states(&self) -> &HashMap<String, DeploymentState> {
        &self.states
    }

    pub fn set_candidate(&mut self, app_name: &str, image: String) {
        self.states
            .entry(app_name.to_string())
            .or_default()
            .candidate_image = Some(image);
    }

    pub fn migrate_legacy(
        &mut self,
        app_name: &str,
        container_name: &str,
        legacy_image_exists: bool,
    ) -> bool {
        if !legacy_image_exists {
            return false;
        }

        let state = self.states.entry(app_name.to_string()).or_default();
        if state.active_image.is_some() || state.active_container.is_some() {
            return false;
        }

        state.active_image = Some(format!("{}:latest", app_name));
        state.active_container = Some(container_name.to_string());
        true
    }

    pub fn activate(&mut self, app_name: &str, image: String, container: String) -> Result<()> {
        let state = self.states.entry(app_name.to_string()).or_default();
        if state.candidate_image.as_deref() != Some(image.as_str()) {
            return Err(Error::State(format!(
                "应用 '{}' 的镜像 '{}' 不是候选镜像",
                app_name, image
            )));
        }
        state.previous_image = state.active_image.replace(image);
        state.previous_container = state.active_container.replace(container);
        state.candidate_image = None;
        Ok(())
    }

    pub fn rollback(&mut self, app_name: &str) -> Result<(String, String)> {
        let state = self
            .states
            .get_mut(app_name)
            .ok_or_else(|| Error::State(format!("应用 '{}' 没有部署状态", app_name)))?;
        let image = state
            .previous_image
            .clone()
            .ok_or_else(|| Error::State(format!("应用 '{}' 没有可回滚镜像", app_name)))?;
        let container = state
            .previous_container
            .clone()
            .ok_or_else(|| Error::State(format!("应用 '{}' 没有可回滚容器", app_name)))?;
        std::mem::swap(&mut state.active_image, &mut state.previous_image);
        std::mem::swap(&mut state.active_container, &mut state.previous_container);
        Ok((image, container))
    }
}

pub fn deployment_state_path(state_file_path: &str) -> PathBuf {
    PathBuf::from(format!("{}.deployments.yml", state_file_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_reference_uses_immutable_short_hash() {
        assert_eq!(
            image_reference("api", "0123456789abcdef"),
            "api:sha-0123456789ab"
        );
    }

    #[test]
    fn test_candidate_container_name_uses_image_version() {
        assert_eq!(
            candidate_container_name("api", "api:sha-0123456789ab"),
            "api--0123456789ab"
        );
    }

    #[test]
    fn test_activate_records_previous_version() {
        let mut store = DeploymentStore::new("unused.yml");
        store.migrate_legacy("api", "api", true);
        store.set_candidate("api", "api:sha-new".to_string());
        store
            .activate("api", "api:sha-new".to_string(), "api--new".to_string())
            .unwrap();
        let state = store.state("api").unwrap();
        assert_eq!(state.active_image.as_deref(), Some("api:sha-new"));
        assert_eq!(state.previous_image.as_deref(), Some("api:latest"));
    }

    #[test]
    fn test_activate_rejects_non_candidate_image() {
        let mut store = DeploymentStore::new("unused.yml");
        assert!(store
            .activate("api", "api:sha-nope".to_string(), "api--nope".to_string())
            .is_err());
    }

    #[test]
    fn test_migrate_legacy_preserves_candidate_and_registers_running_legacy_app() {
        let mut store = DeploymentStore::new("unused.yml");
        store.set_candidate("api", "api:sha-candidate".to_string());

        assert!(store.migrate_legacy("api", "api-container", true));

        let state = store.state("api").unwrap();
        assert_eq!(state.active_image.as_deref(), Some("api:latest"));
        assert_eq!(state.active_container.as_deref(), Some("api-container"));
        assert_eq!(
            state.candidate_image.as_deref(),
            Some("api:sha-candidate")
        );
    }

    #[test]
    fn test_rollback_swaps_active_and_previous() {
        let mut store = DeploymentStore::new("unused.yml");
        store.migrate_legacy("api", "api", true);
        store.set_candidate("api", "api:sha-new".to_string());
        store
            .activate("api", "api:sha-new".to_string(), "api--new".to_string())
            .unwrap();
        assert_eq!(
            store.rollback("api").unwrap(),
            ("api:latest".to_string(), "api".to_string())
        );
        assert_eq!(
            store.state("api").unwrap().active_image.as_deref(),
            Some("api:latest")
        );
    }

    #[test]
    fn test_active_runtime_apps_selects_active_image_and_container() {
        let app = AppConfig {
            name: "api".to_string(),
            routes: vec![],
            container_name: "configured-name".to_string(),
            container_port: 3000,
            healthcheck_path: "/".to_string(),
            app_type: crate::config::AppType::Api,
            description: None,
            nginx_extra_config: None,
            path: None,
            docker_volumes: vec![],
            run_as_user: None,
            proxy_connect_timeout: None,
            proxy_read_timeout: None,
            proxy_send_timeout: None,
        };
        let mut states = HashMap::new();
        states.insert(
            "api".to_string(),
            DeploymentState {
                active_image: Some("api:sha-abc".to_string()),
                active_container: Some("api--abc".to_string()),
                ..DeploymentState::default()
            },
        );
        let (apps, images) = active_runtime_apps(&[app], &states).unwrap();
        assert_eq!(apps[0].container_name, "api--abc");
        assert_eq!(images["api"], "api:sha-abc");
    }

    #[test]
    fn test_runtime_apps_with_candidate_allows_first_deployment() {
        let app = AppConfig {
            name: "api".to_string(),
            routes: vec![],
            container_name: "api".to_string(),
            container_port: 3000,
            healthcheck_path: "/".to_string(),
            app_type: crate::config::AppType::Api,
            description: None,
            nginx_extra_config: None,
            path: None,
            docker_volumes: vec![],
            run_as_user: None,
            proxy_connect_timeout: None,
            proxy_read_timeout: None,
            proxy_send_timeout: None,
        };
        let (apps, images, container) =
            runtime_apps_with_candidate(&[app], &HashMap::new(), "api", "api:sha-abc").unwrap();
        assert_eq!(apps[0].container_name, "api--abc");
        assert_eq!(images["api"], "api:sha-abc");
        assert_eq!(container, "api--abc");
    }
}
