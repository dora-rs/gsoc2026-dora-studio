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
          <div v-for="joint in joints" :key="joint.name" class="joint-row">
            <label>{{ joint.name }}</label>
            <div class="joint-input">
              <input type="text" :value="joint.value" disabled />
              <span>rad</span>
            </div>
          </div>
        </div>
      </div>

      <div class="motion-section">
        <div class="motion-section-header">
          <h3>End-Effector Pose</h3>
        </div>
        <div class="pose-grid">
          <div class="pose-row" v-for="row in poseRows" :key="row.label">
            <label>{{ row.label }}</label>
            <input v-for="(val, i) in row.values" :key="i" type="text" :value="val" disabled />
          </div>
        </div>
      </div>

      <div class="motion-section">
        <div class="motion-section-header">
          <h3>Planning Scene</h3>
          <span class="pill">3 objects</span>
        </div>
        <div class="scene-list">
          <div v-for="obj in sceneObjects" :key="obj.name" class="scene-object">
            <span :class="['scene-icon', obj.shape]"></span>
            <div>
              <strong>{{ obj.name }}</strong>
              <small>{{ obj.dims }}</small>
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
      </div>

      <div class="motion-section">
        <div class="motion-section-header">
          <h3>Trajectory Preview</h3>
          <span class="pill">0 waypoints</span>
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
                  规划后将在此显示轨迹路径点
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
  getMoveitStatus,
  getRobotProfile,
  type ApiSource,
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

const moveit = ref<MoveitStatusResponse>({
  installed: false, running: false,
  message: 'Backend API is not connected.',
})
const robotProfileData = ref<RobotProfileResponse>(fallbackRobotProfile)
const robotProfileSource = ref<ApiSource>('fallback')

const robotProfile = computed(() => robotProfileData.value.profile)
const armModules = computed(() => robotProfile.value.modules.filter((module) => module.kind === 'arm'))
const cameraModules = computed(() => robotProfile.value.modules.filter((module) => module.kind === 'camera'))
const optionalModules = computed(() => robotProfile.value.modules.filter((module) => !module.required))
const mappedModuleCount = computed(() => robotProfile.value.modules.filter((module) => module.linkedDisplays.length > 0).length)

const joints = [
  { name: 'shoulder_pan', value: '0.00' },
  { name: 'shoulder_lift', value: '0.00' },
  { name: 'elbow', value: '0.00' },
  { name: 'wrist_1', value: '0.00' },
  { name: 'wrist_2', value: '0.00' },
  { name: 'wrist_3', value: '0.00' },
]

const poseRows = [
  { label: 'XYZ', values: ['0.00', '0.00', '0.00'] },
  { label: 'Quat', values: ['0.00', '0.00', '0.00', '1.00'] },
]

const sceneObjects = [
  { name: 'table', shape: 'box', dims: '0.8 × 1.2 × 0.05' },
  { name: 'obstacle_A', shape: 'sphere', dims: 'radius 0.15' },
  { name: 'target_box', shape: 'cylinder', dims: 'r=0.1 h=0.2' },
]

const goalJoints = ['0.00', '0.00', '0.00', '0.00', '0.00', '0.00']

onMounted(async () => {
  const [moveitResult, robotProfileResult] = await Promise.all([
    getMoveitStatus({
      installed: false, running: false,
      message: 'Backend API is not connected.',
    }),
    getRobotProfile(fallbackRobotProfile),
  ])

  moveit.value = moveitResult.data
  robotProfileData.value = robotProfileResult.data
  robotProfileSource.value = robotProfileResult.source
})
</script>
