use std::collections::HashMap;


use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub projects: Vec<ProjectConfig>,
    pub clients: Vec<Client>,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub breaks: BreakConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default)]
    pub windows: Vec<String>,
    #[serde(default)]
    pub clients: Vec<ProjectClient>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectClient {
    pub name: String,
    #[serde(default="default_ratio")]
    pub ratio: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Client {
    pub name: String,
    #[serde(default)]
    pub data: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LoggingConfig {
    #[serde(alias="windowChange")]
    pub window_change: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BreakConfig {
    #[serde(default)]
    pub windows: Vec<String>,
    #[serde(default,alias="autoResume")]
    pub auto_resume: bool
}

fn default_ratio() -> f64 { 1. }

#[cfg(test)]
mod tests {
    use config::{File, FileFormat};

    use super::*;

    #[test]
    fn test_parse() {
        let cfg = config::Config::builder().add_source(File::from_str("
projects:
  - name: Xorcery
    windows:
      - Windows PowerShell
      - \"track-work\"
    clients:
      - name: XO
  - name: EKS
    windows:
      - \"@davidgiga1993 - Discord\"
    clients:
      - name: Innovation
        ratio: 0.75
      - name: Maintenance
        ratio: 0.25
  - name: Swag

clients:
  - name: XO
    data:
      psp: IT.1
  - name: Innovation
    data:
      psp: IT.2
  - name: Maintenance
    data:
      psp: IT.3
  - name: Without data

breaks:
  windows:
    - Test
  autoResume: true

logging:
  windowChange: true", FileFormat::Yaml)).build();
        let app_cfg: AppConfig = cfg.unwrap().try_deserialize().expect("should be a valid config");
        assert_eq!(app_cfg.projects[0].name, "Xorcery");
        assert_eq!(app_cfg.projects[0].windows[0], "Windows PowerShell");
        assert_eq!(app_cfg.projects[0].clients[0].name, "XO");
        assert_eq!(app_cfg.projects[0].clients[0].ratio, 1.);
        assert_eq!(app_cfg.projects[1].clients[0].name, "Innovation");
        assert_eq!(app_cfg.projects[1].clients[0].ratio, 0.75);
        assert_eq!(app_cfg.clients[0].name, "XO");
        assert_eq!(app_cfg.clients[0].data["psp"], "IT.1");
        assert_eq!(app_cfg.breaks.windows[0], "Test");
        assert!(app_cfg.breaks.auto_resume);
    }
}