use crate::models::{
    DvizDisplay, DvizDisplaysResponse, DvizSnapshotResponse, DvizSnapshotSummary, DvizStatus,
    DvizTopic, DvizTopicsResponse, MoveitStatus, RobotModule, RobotProfile, RobotProfileResponse,
    RobotWorkflow,
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

pub fn query_dviz_snapshot() -> DvizSnapshotResponse {
    let status = query_dviz();
    dviz_snapshot_response(status)
}

fn dviz_snapshot_response(status: DvizStatus) -> DvizSnapshotResponse {
    let topics = dviz_topics_response(&status);
    let displays = dviz_displays_response(&status);
    let summary = DvizSnapshotSummary {
        topic_count: topics.topics.len(),
        ready_topic_count: topics
            .topics
            .iter()
            .filter(|topic| topic.status == "ready")
            .count(),
        idle_topic_count: topics
            .topics
            .iter()
            .filter(|topic| topic.status == "idle")
            .count(),
        display_count: displays.displays.len(),
        enabled_display_count: displays
            .displays
            .iter()
            .filter(|display| display.enabled)
            .count(),
    };

    DvizSnapshotResponse {
        source: topics.source,
        message: topics.message,
        status,
        summary,
    }
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

pub fn query_robot_profile() -> RobotProfileResponse {
    RobotProfileResponse {
        source: "demo".to_string(),
        message: "Showing a capability-first robot profile until live robot registry integration is available.".to_string(),
        profile: demo_robot_profile(),
    }
}

fn demo_robot_profile() -> RobotProfile {
    RobotProfile {
        id: "nano-so101-family".to_string(),
        name: "Nano SO101 Family".to_string(),
        family: "nano manipulator platform".to_string(),
        summary: "Adaptable profile for SO101-style arms, multiple cameras, optional mobility, and optional lidar.".to_string(),
        simulation_owner: "dora-moveit2 / MuJoCo".to_string(),
        viewport_role: "Studio mirrors moveit-side simulated state; it does not own simulation.".to_string(),
        modules: vec![
            RobotModule {
                id: "left-arm".to_string(),
                name: "Left SO101 Arm".to_string(),
                kind: "arm".to_string(),
                role: "manipulation".to_string(),
                transport: "dora dataflow".to_string(),
                frame: "left_arm_base".to_string(),
                status: "ready".to_string(),
                summary: "Primary manipulator slot with gripper-ready joint state.".to_string(),
                required: true,
                source_topics: vec!["/robot/model".to_string(), "/world/tf".to_string()],
                linked_displays: vec!["robotmodel".to_string(), "tf".to_string()],
            },
            RobotModule {
                id: "right-arm".to_string(),
                name: "Right SO101 Arm".to_string(),
                kind: "arm".to_string(),
                role: "manipulation".to_string(),
                transport: "dora dataflow".to_string(),
                frame: "right_arm_base".to_string(),
                status: "optional".to_string(),
                summary: "Second manipulator slot enabled by robot profile data.".to_string(),
                required: false,
                source_topics: vec!["/robot/model".to_string(), "/world/tf".to_string()],
                linked_displays: vec!["robotmodel".to_string(), "tf".to_string()],
            },
            RobotModule {
                id: "camera-array".to_string(),
                name: "OpenCV Camera Array".to_string(),
                kind: "camera".to_string(),
                role: "perception / recording".to_string(),
                transport: "OpenCV node".to_string(),
                frame: "camera_mounts".to_string(),
                status: "ready".to_string(),
                summary: "Variable camera count; current target supports up to four camera slots.".to_string(),
                required: true,
                source_topics: vec!["/world/points".to_string(), "/world/markers".to_string()],
                linked_displays: vec!["pointcloud".to_string(), "markers".to_string()],
            },
            RobotModule {
                id: "rgbd-camera".to_string(),
                name: "RGB-D Camera".to_string(),
                kind: "camera".to_string(),
                role: "depth perception".to_string(),
                transport: "Orbbec / RGB-D node".to_string(),
                frame: "depth_camera".to_string(),
                status: "optional".to_string(),
                summary: "Optional depth stream for point cloud and scene preview displays.".to_string(),
                required: false,
                source_topics: vec!["/world/points".to_string()],
                linked_displays: vec!["pointcloud".to_string()],
            },
            RobotModule {
                id: "mobile-base".to_string(),
                name: "Mobile Base".to_string(),
                kind: "mobility".to_string(),
                role: "navigation".to_string(),
                transport: "profile slot".to_string(),
                frame: "base_link".to_string(),
                status: "optional".to_string(),
                summary: "Reserved interface for base control once the control path is verified.".to_string(),
                required: false,
                source_topics: vec!["/world/tf".to_string(), "/world/markers".to_string()],
                linked_displays: vec!["tf".to_string(), "markers".to_string()],
            },
            RobotModule {
                id: "lidar".to_string(),
                name: "Lidar".to_string(),
                kind: "sensor".to_string(),
                role: "scan / mapping".to_string(),
                transport: "profile slot".to_string(),
                frame: "lidar_link".to_string(),
                status: "optional".to_string(),
                summary: "Reserved LaserScan source for dviz display linking.".to_string(),
                required: false,
                source_topics: vec!["/world/laser".to_string()],
                linked_displays: vec!["laserscan".to_string()],
            },
        ],
        workflows: vec![
            RobotWorkflow {
                id: "teleop".to_string(),
                name: "Teleoperation".to_string(),
                status: "planned".to_string(),
                owner: "dorobot dataflow".to_string(),
                summary: "Manual control path for SO101-style robot operation.".to_string(),
            },
            RobotWorkflow {
                id: "recording".to_string(),
                name: "Data Collection".to_string(),
                status: "planned".to_string(),
                owner: "dorobot dataflow".to_string(),
                summary: "Camera and robot-state recording workflow for dataset capture.".to_string(),
            },
            RobotWorkflow {
                id: "inference".to_string(),
                name: "Inference".to_string(),
                status: "planned".to_string(),
                owner: "dorobot dataflow".to_string(),
                summary: "Policy-driven operation once model runtime integration is available.".to_string(),
            },
            RobotWorkflow {
                id: "planning".to_string(),
                name: "Motion Planning".to_string(),
                status: "planned".to_string(),
                owner: "dora-moveit2".to_string(),
                summary: "IK, planning, execution, and MuJoCo state simulation stay moveit-owned.".to_string(),
            },
        ],
        visualization_displays: vec![
            "RobotModel".to_string(),
            "TF Frames".to_string(),
            "PointCloud".to_string(),
            "LaserScan".to_string(),
            "Markers".to_string(),
        ],
        planning_capabilities: vec![
            "robot config selection".to_string(),
            "IK readiness".to_string(),
            "trajectory preview".to_string(),
            "moveit-owned MuJoCo state mirror".to_string(),
        ],
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

    #[test]
    fn demo_dviz_snapshot_counts_topics_and_displays() {
        let snapshot = dviz_snapshot_response(dviz_status(false));

        assert_eq!(snapshot.source, "demo");
        assert_eq!(snapshot.summary.topic_count, 5);
        assert_eq!(snapshot.summary.ready_topic_count, 2);
        assert_eq!(snapshot.summary.idle_topic_count, 3);
        assert_eq!(snapshot.summary.display_count, 7);
        assert_eq!(snapshot.summary.enabled_display_count, 3);
    }

    #[test]
    fn demo_robot_profile_keeps_simulation_moveit_owned() {
        let response = query_robot_profile();

        assert_eq!(response.source, "demo");
        assert!(response.profile.simulation_owner.contains("dora-moveit2"));
        assert!(response
            .profile
            .viewport_role
            .contains("does not own simulation"));
        assert!(response
            .profile
            .modules
            .iter()
            .any(|module| module.kind == "arm" && module.required));
        assert!(response
            .profile
            .visualization_displays
            .iter()
            .any(|display| display == "RobotModel"));
    }

    #[test]
    fn demo_robot_modules_reference_dviz_topics_and_displays() {
        let topics = dviz_topics_response(&dviz_status(false))
            .topics
            .into_iter()
            .map(|topic| topic.name)
            .collect::<Vec<_>>();
        let displays = dviz_displays_response(&dviz_status(false))
            .displays
            .into_iter()
            .map(|display| display.id)
            .collect::<Vec<_>>();
        let profile = demo_robot_profile();

        for module in profile.modules {
            assert!(!module.source_topics.is_empty());
            assert!(!module.linked_displays.is_empty());

            for topic in module.source_topics {
                assert!(topics.contains(&topic));
            }

            for display in module.linked_displays {
                assert!(displays.contains(&display));
            }
        }
    }
}
