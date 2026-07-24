<template>
  <section class="viz-layout">
    <aside class="viz-panel viz-left">
      <div class="viz-panel-header">
        <h2>Data Sources</h2>
        <div class="viz-panel-actions">
          <span :class="['pill', backendConnected ? 'success' : 'stopped']">
            {{ visualizationDataLabel }}
          </span>
          <button class="viz-refresh-button" type="button" :disabled="isRefreshing" @click="loadVisualizationData">
            {{ refreshLabel }}
          </button>
        </div>
      </div>

      <div class="viz-search">
        <input v-model="topicSearch" type="text" placeholder="Filter topics by name, type, status..." />
      </div>

      <div class="robot-profile-card">
        <div class="robot-profile-title">
          <div>
            <span>Robot Profile</span>
            <strong>{{ robotProfile.name }}</strong>
          </div>
          <button class="robot-profile-toggle" type="button" @click="robotModulesExpanded = !robotModulesExpanded">
            {{ robotModulesExpanded ? 'Hide modules' : `Show ${robotProfile.modules.length} modules` }}
          </button>
        </div>
        <p>{{ robotProfile.summary }}</p>
        <div v-if="selectedRobotModule && !robotModulesExpanded" class="robot-module-summary">
          <span :class="['robot-module-dot', selectedRobotModule.status]"></span>
          <strong>{{ selectedRobotModule.name }}</strong>
          <small>{{ selectedRobotModule.linkedDisplays.length }} displays · {{ selectedRobotModule.sourceTopics.length }} topics</small>
        </div>
        <div v-if="robotModulesExpanded" class="robot-module-list">
          <button
            v-for="module in visibleRobotModules"
            :key="module.id"
            :class="['robot-module-chip', { selected: selectedRobotModule?.id === module.id }]"
            type="button"
            @click="selectRobotModule(module.id)"
          >
            <span :class="['robot-module-dot', module.status]"></span>
            <div>
              <strong>{{ module.name }}</strong>
              <small>{{ module.kind }} · {{ module.status }}</small>
            </div>
          </button>
        </div>
      </div>

      <div class="display-list">
        <div
          v-for="display in displayData.displays"
          :key="display.id"
          :class="['display-item', { enabled: display.enabled, inactive: !display.enabled, linked: moduleLinkedDisplayIds.has(display.id) }]"
        >
          <input type="checkbox" :checked="display.enabled" @change="toggleDisplay(display.id)" />
          <span :class="['display-dot', display.color]"></span>
          <button class="display-label" type="button" :disabled="!display.sourceTopic" @click="inspectDisplayTopic(display)">
            <strong>{{ display.name }}</strong>
            <small>{{ display.summary }}</small>
          </button>
          <span :class="['display-status', display.status]">{{ display.status }}</span>
        </div>
      </div>

      <div class="viz-section">
        <div class="viz-section-header">
          <h3>Topic Preview</h3>
          <span class="viz-section-source">{{ filteredTopics.length }}/{{ topicData.topics.length }} · {{ topicData.source }}</span>
        </div>
        <button
          v-for="topic in filteredTopics"
          :key="topic.name"
          :class="['topic-preview-box', { selected: selectedTopic?.name === topic.name, linked: moduleSourceTopicNames.has(topic.name) }]"
          type="button"
          @click="selectTopic(topic.name)"
        >
          <div class="topic-title">
            <code>{{ topic.name }}</code>
            <span :class="['topic-status', topic.status]">{{ topic.status }}</span>
          </div>
          <span>{{ topic.dataType }} · {{ topic.summary }}</span>
          <small>{{ topic.source }} · {{ topic.messageRateHz }} Hz · {{ topic.lastSeen }}</small>
        </button>
        <p v-if="filteredTopics.length === 0" class="viz-empty-state">
          No topics match the current filter.
        </p>
      </div>

      <div :class="['viz-data-state', dataStateKind]">
        <strong>{{ dataStateTitle }}</strong>
        <span>{{ dataStateMessage }}</span>
      </div>

      <div class="viz-status-row">
        <span class="status-dot" :class="backendConnected ? 'on' : 'off'"></span>
        <span>{{ connectionLabel }}</span>
        <span class="viz-version">{{ refreshMessage }}</span>
      </div>
    </aside>

    <article class="viz-panel viz-center">
      <div class="viz-panel-header">
        <h2>3D Viewport</h2>
        <span class="pill">metadata preview</span>
      </div>
      <div class="viewport-placeholder">
        <div class="viewport-grid-bg"></div>
        <div class="viewport-center">
          <div class="viewport-icon">{{ selectedTopicInitial }}</div>
          <strong>{{ viewportTitle }}</strong>
          <p>{{ viewportSubtitle }}</p>
          <div class="viewport-summary-grid">
            <div class="viewport-summary-item">
              <strong>{{ enabledDisplays.length }}/{{ snapshotSummary.displayCount }}</strong>
              <span>enabled displays</span>
            </div>
            <div class="viewport-summary-item">
              <strong>{{ robotProfile.modules.length }}</strong>
              <span>profile modules</span>
            </div>
            <div class="viewport-summary-item">
              <strong>{{ readyTopicCount }}/{{ snapshotSummary.topicCount }}</strong>
              <span>ready topics</span>
            </div>
          </div>
          <div class="viewport-boundary-note">
            <span>Simulation owner</span>
            <strong>{{ robotProfile.simulationOwner }}</strong>
            <small>{{ robotProfile.viewportRole }}</small>
          </div>
          <span class="viewport-hint">{{ viewportHint }}</span>
        </div>
      </div>
      <div class="viewport-controls">
        <button type="button" :disabled="!selectedTopic" @click="clearTopicSelection">Clear selection</button>
        <button type="button" @click="showReadyTopics">Ready topics</button>
        <button type="button" @click="showAllTopics">All topics</button>
        <span class="fps-indicator">metadata mode</span>
      </div>
    </article>

    <aside class="viz-panel viz-right">
      <div class="viz-panel-header">
        <h2>Inspector</h2>
        <span v-if="selectedTopic" :class="['topic-status', selectedTopic.status]">{{ selectedTopic.status }}</span>
      </div>

      <div class="interaction-guide-card">
        <strong>Interaction Workflow</strong>
        <ul>
          <li>Select a topic to inspect metadata.</li>
          <li>Toggle displays locally to update the viewport summary.</li>
          <li>Refresh to reload backend dviz metadata and snapshot counts.</li>
        </ul>
      </div>

      <div class="snapshot-summary-card">
        <div>
          <span>Snapshot Source</span>
          <strong>{{ snapshotData.source }} · {{ snapshotApiSource }}</strong>
        </div>
        <div class="snapshot-summary-grid">
          <span>{{ snapshotSummary.topicCount }} topics</span>
          <span>{{ snapshotSummary.displayCount }} displays</span>
          <span>{{ snapshotSummary.enabledDisplayCount }} enabled</span>
        </div>
      </div>

      <div class="robot-inspector-card">
        <div class="robot-inspector-header">
          <span>Robot Stack</span>
          <strong>{{ robotProfileData.source }} · {{ robotProfileApiSource }}</strong>
        </div>
        <div class="detail-row compact">
          <span>Family</span>
          <strong>{{ robotProfile.family }}</strong>
        </div>
        <div class="detail-row compact">
          <span>Viewport Role</span>
          <strong>mirror only</strong>
        </div>
        <div v-if="selectedRobotModule" class="robot-module-detail">
          <strong>{{ selectedRobotModule.name }}</strong>
          <p>{{ selectedRobotModule.summary }}</p>
          <div class="robot-display-tags">
            <span v-for="topic in selectedRobotModule.sourceTopics" :key="topic">{{ topic }}</span>
          </div>
          <div class="robot-display-tags">
            <span v-for="display in selectedRobotModule.linkedDisplays" :key="display">{{ display }}</span>
          </div>
        </div>
        <div v-else class="robot-display-tags">
          <span v-for="display in robotProfile.visualizationDisplays" :key="display">{{ display }}</span>
        </div>
        <div class="robot-workflow-list">
          <div v-for="workflow in robotProfile.workflows" :key="workflow.id" class="robot-workflow-item">
            <strong>{{ workflow.name }}</strong>
            <small>{{ workflow.owner }} · {{ workflow.status }}</small>
          </div>
        </div>
      </div>

      <div v-if="selectedTopic" class="topic-detail-card">
        <div class="topic-detail-title">
          <span>Selected Topic</span>
          <code>{{ selectedTopic.name }}</code>
        </div>
        <div class="detail-row">
          <span>Data Type</span>
          <strong>{{ selectedTopic.dataType }}</strong>
        </div>
        <div class="detail-row">
          <span>Source</span>
          <strong>{{ selectedTopic.source }}</strong>
        </div>
        <div class="detail-row">
          <span>Rate</span>
          <strong>{{ selectedTopic.messageRateHz }} Hz</strong>
        </div>
        <div class="detail-row">
          <span>Last Seen</span>
          <strong>{{ selectedTopic.lastSeen }}</strong>
        </div>
        <p>{{ selectedTopic.summary }}</p>

        <div class="linked-display-section">
          <div class="linked-display-header">
            <span>Linked Displays</span>
            <strong>{{ linkedDisplays.length }}</strong>
          </div>
          <div v-for="display in linkedDisplays" :key="display.id" class="linked-display-item">
            <span :class="['display-dot', display.color]"></span>
            <div>
              <strong>{{ display.name }}</strong>
              <small>{{ display.enabled ? 'enabled' : 'disabled' }} · {{ display.status }}</small>
            </div>
            <button type="button" @click="toggleDisplay(display.id)">
              {{ display.enabled ? 'Disable' : 'Enable' }}
            </button>
          </div>
          <small v-if="linkedDisplays.length === 0" class="linked-display-empty">
            No display currently references this topic.
          </small>
        </div>
      </div>
      <div v-else class="topic-detail-card empty">
        <strong>No topic selected</strong>
        <p>Select a topic from the preview list to inspect its metadata.</p>
      </div>

      <div class="prop-group">
        <div class="prop-group-header">
          <span class="display-dot blue"></span>
          <strong>Enabled Displays</strong>
          <span class="prop-badge">{{ enabledDisplays.length }} active</span>
        </div>
        <div v-for="display in enabledDisplays" :key="display.id" class="detail-row compact">
          <span>{{ display.name }}</span>
          <strong>{{ display.sourceTopic || 'viewport' }}</strong>
        </div>
        <span v-if="enabledDisplays.length === 0" class="prop-value muted">No displays enabled</span>
      </div>

      <div v-for="group in propertyGroups" :key="group.name" class="prop-group">
        <div class="prop-group-header">
          <span :class="['display-dot', group.color]"></span>
          <strong>{{ group.name }}</strong>
        </div>

        <div v-for="prop in group.props" :key="prop.label" class="prop-row">
          <label>{{ prop.label }}</label>
          <div class="prop-control">
            <input
              v-if="prop.type === 'slider'"
              type="range"
              :value="prop.value"
              :min="prop.min"
              :max="prop.max"
              disabled
            />
            <span v-if="prop.type === 'slider'" class="prop-value">{{ prop.value }}</span>
            <div v-else-if="prop.type === 'color'" class="prop-color">
              <span class="color-swatch" :style="{ background: prop.value }"></span>
              <span>{{ prop.value }}</span>
            </div>
          </div>
        </div>
      </div>
    </aside>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import {
  getDvizDisplays,
  getDvizSnapshot,
  getDvizStatus,
  getDvizTopics,
  getRobotProfile,
  type ApiResult,
  type ApiSource,
  type DvizDisplayResponse,
  type DvizDisplaysResponse,
  type DvizSnapshotResponse,
  type DvizStatusResponse,
  type DvizTopicResponse,
  type DvizTopicsResponse,
  type RobotModuleResponse,
  type RobotProfileResponse,
} from '../api'

const fallbackDvizStatus: DvizStatusResponse = {
  installed: false,
  running: false,
  binaryPath: null,
  message: 'Backend API is not connected.',
}

const fallbackTopicData: DvizTopicsResponse = {
  source: 'frontend fallback',
  message: 'Backend API is not connected; showing frontend fallback topics.',
  topics: [
    {
      name: '/world/points',
      dataType: 'PointCloud',
      source: 'frontend fallback',
      status: 'ready',
      messageRateHz: 12,
      lastSeen: 'demo frame',
      summary: '1,024 pts',
    },
    {
      name: '/world/tf',
      dataType: 'Transform3D',
      source: 'frontend fallback',
      status: 'ready',
      messageRateHz: 30,
      lastSeen: 'demo frame',
      summary: '5 frames',
    },
  ],
}

const fallbackDisplayData: DvizDisplaysResponse = {
  source: 'frontend fallback',
  message: 'Backend API is not connected; showing frontend fallback displays.',
  displays: [
    {
      id: 'grid',
      name: 'Grid',
      dataType: 'Viewport',
      enabled: true,
      sourceTopic: null,
      status: 'ready',
      summary: 'Ground grid · 10×10',
      color: 'gray',
    },
    {
      id: 'axes',
      name: 'Axes',
      dataType: 'Viewport',
      enabled: true,
      sourceTopic: null,
      status: 'ready',
      summary: 'RGB axes · 1.0m',
      color: 'red',
    },
    {
      id: 'tf',
      name: 'TF Frames',
      dataType: 'Transform3D',
      enabled: true,
      sourceTopic: '/world/tf',
      status: 'ready',
      summary: 'Frame tree · 5 active frames',
      color: 'green',
    },
    {
      id: 'pointcloud',
      name: 'PointCloud',
      dataType: 'PointCloud',
      enabled: false,
      sourceTopic: '/world/points',
      status: 'idle',
      summary: 'Point cloud data · color/intensity',
      color: 'blue',
    },
  ],
}

const fallbackSnapshotData: DvizSnapshotResponse = {
  source: 'frontend fallback',
  message: 'Backend API is not connected; using frontend fallback snapshot.',
  status: fallbackDvizStatus,
  summary: {
    topicCount: fallbackTopicData.topics.length,
    readyTopicCount: fallbackTopicData.topics.filter((topic) => topic.status === 'ready').length,
    idleTopicCount: fallbackTopicData.topics.filter((topic) => topic.status === 'idle').length,
    displayCount: fallbackDisplayData.displays.length,
    enabledDisplayCount: fallbackDisplayData.displays.filter((display) => display.enabled).length,
  },
}

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
        id: 'teleop',
        name: 'Teleoperation',
        status: 'planned',
        owner: 'dorobot dataflow',
        summary: 'Manual control path for SO101-style robot operation.',
      },
      {
        id: 'recording',
        name: 'Data Collection',
        status: 'planned',
        owner: 'dorobot dataflow',
        summary: 'Camera and robot-state recording workflow for dataset capture.',
      },
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

const dviz = ref<DvizStatusResponse>(fallbackDvizStatus)
const topicData = ref<DvizTopicsResponse>(fallbackTopicData)
const displayData = ref<DvizDisplaysResponse>(fallbackDisplayData)
const snapshotData = ref<DvizSnapshotResponse>(fallbackSnapshotData)
const robotProfileData = ref<RobotProfileResponse>(fallbackRobotProfile)
const topicApiSource = ref<ApiSource>('fallback')
const statusApiSource = ref<ApiSource>('fallback')
const displayApiSource = ref<ApiSource>('fallback')
const snapshotApiSource = ref<ApiSource>('fallback')
const robotProfileApiSource = ref<ApiSource>('fallback')
const topicSearch = ref('')
const selectedTopicName = ref<string | null>(fallbackTopicData.topics[0]?.name ?? null)
const selectedRobotModuleId = ref<string | null>(fallbackRobotProfile.profile.modules[0]?.id ?? null)
const robotModulesExpanded = ref(false)
const isRefreshing = ref(false)
const refreshError = ref<string | null>(null)

const backendConnected = computed(() => (
  statusApiSource.value === 'connected'
  || topicApiSource.value === 'connected'
  || displayApiSource.value === 'connected'
  || snapshotApiSource.value === 'connected'
  || robotProfileApiSource.value === 'connected'
))

const visualizationDataLabel = computed(() => {
  if (topicApiSource.value === 'connected' && topicData.value.source === 'demo') {
    return 'backend demo'
  }

  if (topicApiSource.value === 'connected') {
    return 'backend data'
  }

  return 'fallback data'
})

const connectionLabel = computed(() => {
  if (!backendConnected.value) {
    return 'backend unavailable'
  }

  if (statusApiSource.value === 'connected') {
    return dviz.value.running ? 'dviz process detected' : 'backend connected'
  }

  return 'backend partially connected'
})

const normalizedTopicSearch = computed(() => topicSearch.value.trim().toLowerCase())

const filteredTopics = computed(() => {
  if (!normalizedTopicSearch.value) {
    return topicData.value.topics
  }

  return topicData.value.topics.filter((topic) => {
    const searchable = [topic.name, topic.dataType, topic.status, topic.source, topic.summary]
    return searchable.some((value) => value.toLowerCase().includes(normalizedTopicSearch.value))
  })
})

const selectedTopic = computed(() => {
  if (!selectedTopicName.value) {
    return null
  }

  return topicData.value.topics.find((topic) => topic.name === selectedTopicName.value) ?? null
})

const robotProfile = computed(() => robotProfileData.value.profile)
const visibleRobotModules = computed(() => robotProfile.value.modules)
const selectedRobotModule = computed(() => {
  if (!selectedRobotModuleId.value) {
    return null
  }

  return robotProfile.value.modules.find((module) => module.id === selectedRobotModuleId.value) ?? null
})
const moduleSourceTopicNames = computed(() => new Set(selectedRobotModule.value?.sourceTopics ?? []))
const moduleLinkedDisplayIds = computed(() => new Set(selectedRobotModule.value?.linkedDisplays ?? []))
const enabledDisplays = computed(() => displayData.value.displays.filter((display) => display.enabled))
const linkedDisplays = computed(() => {
  if (!selectedTopic.value) {
    return []
  }

  return displayData.value.displays.filter((display) => display.sourceTopic === selectedTopic.value?.name)
})
const snapshotSummary = computed(() => snapshotData.value.summary)
const readyTopicCount = computed(() => snapshotSummary.value.readyTopicCount)

const viewportTitle = computed(() => {
  if (!selectedTopic.value) {
    return 'Visualization metadata preview'
  }

  return `${selectedTopic.value.dataType} preview`
})

const viewportSubtitle = computed(() => {
  if (!selectedTopic.value) {
    return 'Select a topic to inspect metadata before live 3D streaming is wired.'
  }

  return `${selectedTopic.value.name} · ${selectedTopic.value.summary}`
})

const viewportHint = computed(() => {
  const modulePrefix = selectedRobotModule.value
    ? `${selectedRobotModule.value.name}: ${moduleLinkedDisplayIds.value.size} displays / ${moduleSourceTopicNames.value.size} topics`
    : 'No robot module selected'

  return `${modulePrefix} · ${enabledDisplays.value.length}/${snapshotSummary.value.displayCount} displays enabled · snapshot ${snapshotApiSource.value}`
})

const selectedTopicInitial = computed(() => selectedTopic.value?.dataType.slice(0, 1).toUpperCase() ?? 'V')
const refreshLabel = computed(() => (isRefreshing.value ? 'Refreshing...' : 'Refresh'))
const refreshMessage = computed(() => refreshError.value ?? topicData.value.message)

const dataStateKind = computed(() => {
  if (isRefreshing.value) {
    return 'loading'
  }

  if (refreshError.value) {
    return 'error'
  }

  return backendConnected.value ? 'connected' : 'fallback'
})

const dataStateTitle = computed(() => {
  if (isRefreshing.value) {
    return 'Refreshing metadata'
  }

  if (refreshError.value) {
    return 'Fallback metadata active'
  }

  return backendConnected.value ? 'Backend metadata connected' : 'Frontend fallback active'
})

const dataStateMessage = computed(() => {
  if (isRefreshing.value) {
    return 'Reloading dviz status, topics, displays, and snapshot counts.'
  }

  if (refreshError.value) {
    return refreshError.value
  }

  return snapshotData.value.message
})

const propertyGroups = [
  {
    name: 'Grid', color: 'gray',
    props: [
      { type: 'slider' as const, label: 'Cell Count', value: '10', min: '2', max: '50' },
      { type: 'slider' as const, label: 'Cell Size', value: '1.0', min: '0.1', max: '10.0' },
      { type: 'color' as const, label: 'Color', value: '#7c8aa5' },
    ],
  },
  {
    name: 'PointCloud', color: 'blue',
    props: [
      { type: 'slider' as const, label: 'Point Size', value: '0.02', min: '0.001', max: '0.5' },
      { type: 'color' as const, label: 'Flat Color', value: '#ffffff' },
      { type: 'slider' as const, label: 'Decay Time', value: '3.0', min: '0', max: '30' },
      { type: 'slider' as const, label: 'Queue Size', value: '10', min: '1', max: '50' },
    ],
  },
  {
    name: 'TF Frames', color: 'green',
    props: [
      { type: 'slider' as const, label: 'Axis Scale', value: '0.3', min: '0.1', max: '5.0' },
      { type: 'slider' as const, label: 'Timeout', value: '5.0', min: '1.0', max: '30.0' },
    ],
  },
]

function selectTopic(name: string) {
  selectedTopicName.value = name
}

function selectRobotModule(id: string) {
  selectedRobotModuleId.value = id
  const module = robotProfile.value.modules.find((item: RobotModuleResponse) => item.id === id)
  const firstTopic = module?.sourceTopics.find((topic) => topicData.value.topics.some((candidate) => candidate.name === topic))

  if (firstTopic) {
    topicSearch.value = ''
    selectedTopicName.value = firstTopic
  }
}

function clearTopicSelection() {
  selectedTopicName.value = null
}

function showReadyTopics() {
  topicSearch.value = 'ready'
}

function showAllTopics() {
  topicSearch.value = ''
}

function toggleDisplay(id: string) {
  displayData.value = {
    ...displayData.value,
    displays: displayData.value.displays.map((display) => (
      display.id === id ? { ...display, enabled: !display.enabled } : display
    )),
  }
}

function inspectDisplayTopic(display: DvizDisplayResponse) {
  if (!display.sourceTopic) {
    return
  }

  topicSearch.value = ''
  selectedTopicName.value = display.sourceTopic
}

function ensureSelectedTopic() {
  if (selectedTopicName.value && topicData.value.topics.some((topic) => topic.name === selectedTopicName.value)) {
    return
  }

  selectedTopicName.value = topicData.value.topics[0]?.name ?? null
}

function ensureSelectedRobotModule() {
  if (selectedRobotModuleId.value && robotProfile.value.modules.some((module) => module.id === selectedRobotModuleId.value)) {
    return
  }

  selectedRobotModuleId.value = robotProfile.value.modules[0]?.id ?? null
}

function firstError(...results: ApiResult<unknown>[]) {
  return results.find((result) => result.source === 'fallback' && result.error)?.error ?? null
}

async function loadVisualizationData() {
  isRefreshing.value = true
  refreshError.value = null

  const [statusResult, topicResult, displayResult, snapshotResult, robotProfileResult] = await Promise.all([
    getDvizStatus(fallbackDvizStatus),
    getDvizTopics(fallbackTopicData),
    getDvizDisplays(fallbackDisplayData),
    getDvizSnapshot(fallbackSnapshotData),
    getRobotProfile(fallbackRobotProfile),
  ])

  dviz.value = snapshotResult.source === 'connected' ? snapshotResult.data.status : statusResult.data
  topicData.value = topicResult.data
  displayData.value = displayResult.data
  snapshotData.value = snapshotResult.data
  robotProfileData.value = robotProfileResult.data
  statusApiSource.value = statusResult.source
  topicApiSource.value = topicResult.source
  displayApiSource.value = displayResult.source
  snapshotApiSource.value = snapshotResult.source
  robotProfileApiSource.value = robotProfileResult.source
  refreshError.value = firstError(statusResult, topicResult, displayResult, snapshotResult, robotProfileResult)
  ensureSelectedRobotModule()
  ensureSelectedTopic()
  isRefreshing.value = false
}

onMounted(loadVisualizationData)
</script>
