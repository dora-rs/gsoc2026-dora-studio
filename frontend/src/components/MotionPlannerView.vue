<template>
  <section class="motion-layout">
    <aside class="motion-panel motion-left">
      <div class="motion-section">
        <div class="motion-section-header">
          <h3>Robot Profile</h3>
          <span class="pill">{{ robotProfileSource }}</span>
        </div>
        <div class="motion-profile-card">
          <strong>{{ robotProfile.name }}</strong>
          <span>{{ robotProfile.family }}</span>
          <p>{{ robotProfile.summary }}</p>
          <div class="motion-profile-meta">
            <span>{{ armModules.length }} arm slots</span>
            <span>{{ cameraModules.length }} camera slots</span>
            <span>{{ optionalModules.length }} optional modules</span>
            <span>{{ mappedModuleCount }} mapped modules</span>
          </div>
        </div>
      </div>

      <div class="motion-section">
        <div class="motion-section-header">
          <h3>Robot State</h3>
          <span class="pill stopped">moveit mirror</span>
        </div>

        <div class="joint-list">
          <div v-for="joint in snapshotJoints" :key="joint.name" class="joint-row">
            <label>{{ joint.name }}</label>
            <div class="joint-input">
              <input type="text" :value="joint.value.toFixed(2)" disabled />
              <span>{{ joint.unit }}</span>
            </div>
            <small>{{ joint.status }} · {{ joint.lowerLimit.toFixed(2) }}..{{ joint.upperLimit.toFixed(2) }}</small>
          </div>
        </div>
      </div>

      <div class="motion-section">
        <div class="motion-section-header">
          <h3>End-Effector Pose</h3>
        </div>
        <div class="pose-grid">
          <div class="pose-row" v-for="row in snapshotPoseRows" :key="row.label">
            <label>{{ row.label }}</label>
            <input v-for="(val, i) in row.values" :key="i" type="text" :value="val" disabled />
          </div>
        </div>
      </div>

      <div class="motion-section">
        <div class="motion-section-header">
          <h3>Planning Scene</h3>
          <span class="pill">{{ moveitSnapshot.scene.objectCount }} objects</span>
        </div>
        <div class="scene-list">
          <div v-for="obj in snapshotSceneObjects" :key="obj.name" class="scene-object">
            <span :class="['scene-icon', obj.shape]"></span>
            <div>
              <strong>{{ obj.name }}</strong>
              <small>{{ obj.dims }} · {{ obj.frame }} · {{ obj.status }}</small>
            </div>
          </div>
        </div>
        <div class="scene-actions">
          <button disabled>+ Add Box</button>
          <button class="secondary" disabled>+ Add Sphere</button>
          <button class="secondary" disabled>− Remove</button>
        </div>
      </div>

      <div class="motion-status">
        <span :class="['status-dot', moveit.running ? 'on' : 'off']"></span>
        {{ moveit.message }}
      </div>
    </aside>

    <article class="motion-panel motion-right">
      <div class="motion-section">
        <div class="motion-section-header">
          <h3>Motion Control</h3>
        </div>

        <div class="goal-tabs">
          <button class="goal-tab active" disabled>Joint Goal</button>
          <button class="goal-tab" disabled>Pose Goal</button>
          <button class="goal-tab" disabled>Cartesian Path</button>
        </div>

        <div class="joint-goal-grid">
          <div v-for="(val, i) in goalJoints" :key="i" class="joint-goal-cell">
            <label>joint_{{ i + 1 }}</label>
            <input type="text" :value="val" disabled />
          </div>
        </div>

        <div class="control-row motion-actions">
          <button class="primary-action" disabled>Plan</button>
          <button class="primary-action execute" disabled>Execute</button>
          <button class="secondary" disabled>Stop</button>
        </div>

        <div class="planner-config">
          <label>Robot Config</label>
          <input type="text" :value="robotProfile.id" disabled />
          <label>Planner</label>
          <select disabled>
            <option>RRT-Connect</option>
            <option>RRT</option>
            <option>PRM</option>
          </select>
          <label>Planning Time</label>
          <input type="text" value="5.0s" disabled />
        </div>

        <div class="motion-boundary-card">
          <span>Simulation Owner</span>
          <strong>{{ robotProfile.simulationOwner }}</strong>
          <p>{{ robotProfile.viewportRole }}</p>
        </div>

        <div class="motion-snapshot-card">
          <span>MoveIt Snapshot</span>
          <strong>{{ snapshotSourceLabel }}</strong>
          <p>{{ moveitSnapshot.message }}</p>
          <div class="motion-profile-meta">
            <span>{{ moveitSnapshot.freshness.status }}</span>
            <span>{{ moveitSnapshot.freshness.lastUpdated }}</span>
            <span>{{ moveitSnapshot.freshness.sourceLabel }}</span>
          </div>
        </div>
      </div>

      <div class="motion-section mujoco-mirror-section">
        <div class="motion-section-header">
          <h3>SO-101 MuJoCo Visual Mirror</h3>
          <span class="pill warning">mirror only</span>
        </div>
        <div class="mujoco-mirror-card">
          <div class="mujoco-stage" :style="visualJointStyle">
            <div class="mujoco-grid-floor"></div>
            <div class="so101-base">
              <span>SO-101</span>
            </div>
            <div class="so101-arm shoulder-link">
              <div class="so101-joint shoulder-joint"></div>
              <div class="so101-arm elbow-link">
                <div class="so101-joint elbow-joint"></div>
                <div class="so101-arm wrist-link">
                  <div class="so101-joint wrist-joint"></div>
                  <div class="so101-gripper">
                    <span class="gripper-finger left"></span>
                    <span class="gripper-finger right"></span>
                  </div>
                </div>
              </div>
            </div>
            <div class="mujoco-target-dot"></div>
          </div>
          <div class="mujoco-mirror-details">
            <span>{{ moveitSnapshot.visualModel.name }}</span>
            <strong>{{ moveitSnapshot.robotConfigId }}</strong>
            <p>{{ moveitSnapshot.viewportRole }}</p>
            <div class="robot-display-tags">
              <span v-for="joint in moveitSnapshot.visualModel.jointOrder" :key="joint">{{ joint }}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="motion-section">
        <div class="motion-section-header">
          <h3>Trajectory Preview</h3>
          <span class="pill">{{ snapshotTrajectory.waypointCount }} waypoints</span>
        </div>
        <div class="trajectory-table-wrap">
          <table class="trajectory-table">
            <thead>
              <tr>
                <th>#</th>
                <th v-for="j in 6" :key="j">joint_{{ j }}</th>
                <th>time</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td colspan="8" class="empty-trajectory">
                  {{ snapshotTrajectory.message }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="motion-section">
        <div class="motion-section-header">
          <h3>IK Solver</h3>
        </div>
        <div class="ik-row">
          <div class="pose-grid">
            <div class="pose-row">
              <label>Position</label>
              <input type="text" value="0.0" disabled />
              <input type="text" value="0.0" disabled />
              <input type="text" value="0.0" disabled />
            </div>
            <div class="pose-row">
              <label>Quaternion</label>
              <input type="text" value="0.0" disabled />
              <input type="text" value="0.0" disabled />
              <input type="text" value="0.0" disabled />
              <input type="text" value="1.0" disabled />
            </div>
          </div>
          <div class="ik-action">
            <button disabled>Solve IK</button>
            <span class="ik-result">—</span>
          </div>
        </div>
      </div>
    </article>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import {
  getMoveitSnapshot,
  getMoveitStatus,
  getRobotProfile,
  type ApiSource,
  type MoveitSnapshotResponse,
  type MoveitStatusResponse,
  type RobotProfileResponse,
} from '../api'

const fallbackRobotProfile: RobotProfileResponse = {
  source: 'frontend fallback',
  message: 'Backend API is not connected; showing the frontend fallback robot profile.',
  profile: {
    id: 'nano-so101-family',
    name: 'Nano SO101 Family',
    family: 'nano manipulator platform',
    summary: 'Capability-first profile for SO101 arms, camera modules, optional base, and optional lidar.',
    simulationOwner: 'dora-moveit2 / MuJoCo',
    viewportRole: 'Studio mirrors moveit-side simulated state; it does not own simulation.',
    modules: [
      {
        id: 'left-arm',
        name: 'Left SO101 Arm',
        kind: 'arm',
        role: 'manipulation',
        transport: 'dora dataflow',
        frame: 'left_arm_base',
        status: 'ready',
        summary: 'Primary manipulator slot with gripper-ready joint state.',
        required: true,
        sourceTopics: ['/robot/model', '/world/tf'],
        linkedDisplays: ['robotmodel', 'tf'],
      },
      {
        id: 'camera-array',
        name: 'OpenCV Camera Array',
        kind: 'camera',
        role: 'perception / recording',
        transport: 'OpenCV node',
        frame: 'camera_mounts',
        status: 'ready',
        summary: 'Variable camera count; target profile supports up to four camera slots.',
        required: true,
        sourceTopics: ['/world/points', '/world/markers'],
        linkedDisplays: ['pointcloud', 'markers'],
      },
      {
        id: 'mobile-base',
        name: 'Mobile Base',
        kind: 'mobility',
        role: 'navigation',
        transport: 'profile slot',
        frame: 'base_link',
        status: 'optional',
        summary: 'Reserved interface for base control once verified.',
        required: false,
        sourceTopics: ['/world/tf', '/world/markers'],
        linkedDisplays: ['tf', 'markers'],
      },
      {
        id: 'lidar',
        name: 'Lidar',
        kind: 'sensor',
        role: 'scan / mapping',
        transport: 'profile slot',
        frame: 'lidar_link',
        status: 'optional',
        summary: 'Reserved LaserScan source for dviz display linking.',
        required: false,
        sourceTopics: ['/world/laser'],
        linkedDisplays: ['laserscan'],
      },
    ],
    workflows: [
      {
        id: 'planning',
        name: 'Motion Planning',
        status: 'planned',
        owner: 'dora-moveit2',
        summary: 'IK, planning, execution, and MuJoCo simulation stay moveit-owned.',
      },
    ],
    visualizationDisplays: ['RobotModel', 'TF Frames', 'PointCloud', 'LaserScan', 'Markers'],
    planningCapabilities: ['robot config selection', 'IK readiness', 'trajectory preview', 'moveit-owned MuJoCo state mirror'],
  },
}

const fallbackMoveitSnapshot: MoveitSnapshotResponse = {
  source: 'frontend fallback',
  message: 'Backend API is not connected; showing a read-only SO-101 MuJoCo mirror fallback.',
  robotProfileId: 'nano-so101-family',
  robotConfigId: 'so101-left-arm-demo',
  simulationOwner: 'dora-moveit2 / MuJoCo',
  viewportRole: 'Studio mirrors moveit-side simulated state; it does not own simulation.',
  freshness: {
    status: 'fallback',
    lastUpdated: 'frontend fallback frame',
    sourceLabel: 'deterministic frontend fallback',
  },
  joints: [
    { name: 'shoulder_pan', value: 0.22, unit: 'rad', lowerLimit: -2.62, upperLimit: 2.62, status: 'ready', source: 'frontend moveit mirror' },
    { name: 'shoulder_lift', value: -0.48, unit: 'rad', lowerLimit: -1.9, upperLimit: 1.9, status: 'ready', source: 'frontend moveit mirror' },
    { name: 'elbow_flex', value: 0.86, unit: 'rad', lowerLimit: -2.1, upperLimit: 2.1, status: 'ready', source: 'frontend moveit mirror' },
    { name: 'wrist_flex', value: -0.34, unit: 'rad', lowerLimit: -1.8, upperLimit: 1.8, status: 'ready', source: 'frontend moveit mirror' },
    { name: 'wrist_roll', value: 0.18, unit: 'rad', lowerLimit: -3.14, upperLimit: 3.14, status: 'ready', source: 'frontend moveit mirror' },
    { name: 'gripper', value: 0.03, unit: 'rad', lowerLimit: 0, upperLimit: 0.8, status: 'ready', source: 'frontend moveit mirror' },
  ],
  endEffectorPose: {
    frame: 'left_gripper_tip',
    position: [0.32, -0.08, 0.24],
    quaternion: [0, 0.18, 0, 0.98],
    source: 'frontend moveit mirror',
  },
  scene: {
    status: 'ready',
    objectCount: 3,
    objects: [
      { name: 'workbench', shape: 'box', dims: '0.80 x 1.20 x 0.05', dimensions: [0.8, 1.2, 0.05], frame: 'world', status: 'fixed' },
      { name: 'pick_target', shape: 'cylinder', dims: 'r=0.04 h=0.12', dimensions: [0.04, 0.12], frame: 'world', status: 'target' },
      { name: 'safety_zone', shape: 'sphere', dims: 'r=0.18', dimensions: [0.18], frame: 'left_arm_base', status: 'collision guard' },
    ],
  },
  trajectory: {
    status: 'idle',
    waypointCount: 0,
    durationSeconds: 0,
    message: 'No plan requested; Studio is showing read-only mirror state.',
  },
  visualModel: {
    modelId: 'so101-mujoco-mirror',
    name: 'SO-101 MuJoCo visual mirror',
    format: 'css-articulated-preview',
    source: 'dora-moveit2 mirror contract',
    jointOrder: ['shoulder_pan', 'shoulder_lift', 'elbow_flex', 'wrist_flex', 'wrist_roll', 'gripper'],
  },
}

const moveit = ref<MoveitStatusResponse>({
  installed: false, running: false,
  message: 'Backend API is not connected.',
})
const robotProfileData = ref<RobotProfileResponse>(fallbackRobotProfile)
const robotProfileSource = ref<ApiSource>('fallback')
const moveitSnapshotData = ref<MoveitSnapshotResponse>(fallbackMoveitSnapshot)
const moveitSnapshotSource = ref<ApiSource>('fallback')

const robotProfile = computed(() => robotProfileData.value.profile)
const armModules = computed(() => robotProfile.value.modules.filter((module) => module.kind === 'arm'))
const cameraModules = computed(() => robotProfile.value.modules.filter((module) => module.kind === 'camera'))
const optionalModules = computed(() => robotProfile.value.modules.filter((module) => !module.required))
const mappedModuleCount = computed(() => robotProfile.value.modules.filter((module) => module.linkedDisplays.length > 0).length)
const moveitSnapshot = computed(() => moveitSnapshotData.value)
const snapshotJoints = computed(() => moveitSnapshot.value.joints)
const snapshotSceneObjects = computed(() => moveitSnapshot.value.scene.objects)
const snapshotTrajectory = computed(() => moveitSnapshot.value.trajectory)
const snapshotPoseRows = computed(() => [
  {
    label: 'XYZ',
    values: moveitSnapshot.value.endEffectorPose.position.map((value) => value.toFixed(2)),
  },
  {
    label: 'Quat',
    values: moveitSnapshot.value.endEffectorPose.quaternion.map((value) => value.toFixed(2)),
  },
])
const visualJointStyle = computed(() => {
  const [base, shoulder, elbow, wrist, roll, gripper] = moveitSnapshot.value.joints
  return {
    '--base-angle': `${(base?.value ?? 0) * 32}deg`,
    '--shoulder-angle': `${-34 + (shoulder?.value ?? 0) * 18}deg`,
    '--elbow-angle': `${42 + (elbow?.value ?? 0) * 16}deg`,
    '--wrist-angle': `${-18 + (wrist?.value ?? 0) * 20}deg`,
    '--roll-angle': `${(roll?.value ?? 0) * 28}deg`,
    '--gripper-open': `${24 + (gripper?.value ?? 0) * 30}px`,
  }
})
const snapshotSourceLabel = computed(() => `${moveitSnapshotSource.value} · ${moveitSnapshot.value.source}`)

const goalJoints = ['0.00', '0.00', '0.00', '0.00', '0.00', '0.00']

onMounted(async () => {
  const [moveitResult, robotProfileResult, moveitSnapshotResult] = await Promise.all([
    getMoveitStatus({
      installed: false, running: false,
      message: 'Backend API is not connected.',
    }),
    getRobotProfile(fallbackRobotProfile),
    getMoveitSnapshot(fallbackMoveitSnapshot),
  ])

  moveit.value = moveitResult.data
  robotProfileData.value = robotProfileResult.data
  robotProfileSource.value = robotProfileResult.source
  moveitSnapshotData.value = moveitSnapshotResult.data
  moveitSnapshotSource.value = moveitSnapshotResult.source
})
</script>
