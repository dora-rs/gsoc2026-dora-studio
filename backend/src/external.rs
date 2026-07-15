use crate::models::{
    DvizDisplay, DvizDisplaysResponse, DvizStatus, DvizTopic, DvizTopicsResponse, MoveitStatus,
};

fn dviz_binary_path() -> Option<std::path::PathBuf> {
    let candidates = ["/home/dora/Desktop/dviz/target/release/dviz"];

    for path in candidates {
        let p = std::path::Path::new(path);
        if p.exists() {
            return Some(p.to_path_buf());
        }
    }

    // Fallback: try to find dviz on PATH
    std::process::Command::new("which")
        .arg("dviz")
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    Some(std::path::PathBuf::from(s))
                } else {
                    None
                }
            } else {
                None
            }
        })
}

pub fn query_dviz() -> DvizStatus {
    match dviz_binary_path() {
        Some(path) => {
            // Check if dviz process is running
            let running = is_process_running("dviz");
            DvizStatus {
                installed: true,
                running,
                binary_path: Some(path.display().to_string()),
                message: if running {
                    "dviz is running and publishing Zenoh data.".to_string()
                } else {
                    "dviz is installed but not running. Start with: cargo run -p dviz-shell --release"
                        .to_string()
                },
            }
        }
        None => DvizStatus {
            installed: false,
            running: false,
            binary_path: None,
            message: "dviz is not installed. Clone from the dviz repository and build with: cargo build --release"
                .to_string(),
        },
    }
}

pub fn query_dviz_topics() -> DvizTopicsResponse {
    dviz_topics_response(&query_dviz())
}

pub fn query_dviz_displays() -> DvizDisplaysResponse {
    dviz_displays_response(&query_dviz())
}

fn dviz_topics_response(status: &DvizStatus) -> DvizTopicsResponse {
    DvizTopicsResponse {
        source: "demo".to_string(),
        message: dviz_metadata_message(status),
        topics: vec![
            DvizTopic {
                name: "/world/points".to_string(),
                data_type: "PointCloud".to_string(),
                source: "backend demo".to_string(),
                status: "ready".to_string(),
                message_rate_hz: 12.0,
                last_seen: "demo frame".to_string(),
                summary: "1,024 colored points for viewport wiring".to_string(),
            },
            DvizTopic {
                name: "/world/tf".to_string(),
                data_type: "Transform3D".to_string(),
                source: "backend demo".to_string(),
                status: "ready".to_string(),
                message_rate_hz: 30.0,
                last_seen: "demo frame".to_string(),
                summary: "5 active frames for TF display wiring".to_string(),
            },
            DvizTopic {
                name: "/world/laser".to_string(),
                data_type: "LaserScan".to_string(),
                source: "backend demo".to_string(),
                status: "idle".to_string(),
                message_rate_hz: 0.0,
                last_seen: "not observed".to_string(),
                summary: "2D scan channel reserved for Zenoh hookup".to_string(),
            },
            DvizTopic {
                name: "/robot/model".to_string(),
                data_type: "RobotModel".to_string(),
                source: "backend demo".to_string(),
                status: "idle".to_string(),
                message_rate_hz: 0.0,
                last_seen: "not observed".to_string(),
                summary: "URDF-style robot model metadata".to_string(),
            },
            DvizTopic {
                name: "/world/markers".to_string(),
                data_type: "Markers".to_string(),
                source: "backend demo".to_string(),
                status: "idle".to_string(),
                message_rate_hz: 0.0,
                last_seen: "not observed".to_string(),
                summary: "Scene annotations for future display controls".to_string(),
            },
        ],
    }
}

fn dviz_displays_response(status: &DvizStatus) -> DvizDisplaysResponse {
    DvizDisplaysResponse {
        source: "demo".to_string(),
        message: dviz_metadata_message(status),
        displays: vec![
            DvizDisplay {
                id: "grid".to_string(),
                name: "Grid".to_string(),
                data_type: "Viewport".to_string(),
                enabled: true,
                source_topic: None,
                status: "ready".to_string(),
                summary: "Ground grid, 10 x 10 cells".to_string(),
                color: "gray".to_string(),
            },
            DvizDisplay {
                id: "axes".to_string(),
                name: "Axes".to_string(),
                data_type: "Viewport".to_string(),
                enabled: true,
                source_topic: None,
                status: "ready".to_string(),
                summary: "RGB coordinate axes at the world origin".to_string(),
                color: "red".to_string(),
            },
            DvizDisplay {
                id: "tf".to_string(),
                name: "TF Frames".to_string(),
                data_type: "Transform3D".to_string(),
                enabled: true,
                source_topic: Some("/world/tf".to_string()),
                status: "ready".to_string(),
                summary: "Frame tree preview from topic metadata".to_string(),
                color: "green".to_string(),
            },
            DvizDisplay {
                id: "pointcloud".to_string(),
                name: "PointCloud".to_string(),
                data_type: "PointCloud".to_string(),
                enabled: false,
                source_topic: Some("/world/points".to_string()),
                status: "idle".to_string(),
                summary: "Point cloud display prepared for Zenoh data".to_string(),
                color: "blue".to_string(),
            },
            DvizDisplay {
                id: "laserscan".to_string(),
                name: "LaserScan".to_string(),
                data_type: "LaserScan".to_string(),
                enabled: false,
                source_topic: Some("/world/laser".to_string()),
                status: "idle".to_string(),
                summary: "2D laser scan display prepared for live data".to_string(),
                color: "orange".to_string(),
            },
            DvizDisplay {
                id: "markers".to_string(),
                name: "Markers".to_string(),
                data_type: "Markers".to_string(),
                enabled: false,
                source_topic: Some("/world/markers".to_string()),
                status: "idle".to_string(),
                summary: "Marker display for scene annotations".to_string(),
                color: "purple".to_string(),
            },
            DvizDisplay {
                id: "robotmodel".to_string(),
                name: "RobotModel".to_string(),
                data_type: "RobotModel".to_string(),
                enabled: false,
                source_topic: Some("/robot/model".to_string()),
                status: "idle".to_string(),
                summary: "Robot model display prepared for URDF metadata".to_string(),
                color: "cyan".to_string(),
            },
        ],
    }
}

fn dviz_metadata_message(status: &DvizStatus) -> String {
    if status.running {
        "dviz process detected; showing API-first demo metadata until Zenoh subscription is wired."
            .to_string()
    } else if status.installed {
        "dviz is installed but not running; showing demo visualization metadata.".to_string()
    } else {
        "dviz is unavailable; showing demo visualization metadata.".to_string()
    }
}

pub fn query_moveit() -> MoveitStatus {
    let installed =
        is_python_package_installed("dora-moveit") || is_python_package_installed("dora_moveit");
    MoveitStatus {
        installed,
        running: false, // Will be updated when coordinator shows moveit nodes
        message: if installed {
            "dora-moveit2 is installed. Start moveit nodes in a Dora dataflow to begin.".to_string()
        } else {
            "dora-moveit2 is not installed. Install with: pip install -e dora_moveit/".to_string()
        },
    }
}

fn is_process_running(name: &str) -> bool {
    std::process::Command::new("pgrep")
        .arg("-x")
        .arg(name)
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

fn is_python_package_installed(name: &str) -> bool {
    std::process::Command::new("pip")
        .args(["show", name])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dviz_status(running: bool) -> DvizStatus {
        DvizStatus {
            installed: true,
            running,
            binary_path: Some("/usr/local/bin/dviz".to_string()),
            message: "dviz test status".to_string(),
        }
    }

    #[test]
    fn demo_dviz_topics_are_stable() {
        let response = dviz_topics_response(&dviz_status(false));

        assert_eq!(response.source, "demo");
        assert_eq!(response.topics.len(), 5);
        assert!(response.message.contains("demo"));
        assert!(response
            .topics
            .iter()
            .any(|topic| topic.name == "/world/points" && topic.data_type == "PointCloud"));
    }

    #[test]
    fn demo_dviz_displays_reference_known_topics() {
        let topics = dviz_topics_response(&dviz_status(false))
            .topics
            .into_iter()
            .map(|topic| topic.name)
            .collect::<Vec<_>>();
        let response = dviz_displays_response(&dviz_status(false));

        assert_eq!(response.source, "demo");
        assert!(response
            .displays
            .iter()
            .any(|display| display.id == "pointcloud" && !display.enabled));

        for topic in response
            .displays
            .iter()
            .filter_map(|display| display.source_topic.as_ref())
        {
            assert!(topics.contains(topic));
        }
    }
}
