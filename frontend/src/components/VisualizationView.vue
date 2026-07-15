<template>
  <section class="viz-layout">
    <aside class="viz-panel viz-left">
      <div class="viz-panel-header">
        <h2>Data Sources</h2>
        <span :class="['pill', backendConnected ? 'success' : 'stopped']">
          {{ visualizationDataLabel }}
        </span>
      </div>

      <div class="viz-search">
        <input type="text" placeholder="搜索 display..." disabled />
      </div>

      <div class="display-list">
        <label
          v-for="display in displayData.displays"
          :key="display.id"
          :class="['display-item', { enabled: display.enabled }]"
        >
          <input type="checkbox" :checked="display.enabled" disabled />
          <span :class="['display-dot', display.color]"></span>
          <div class="display-label">
            <strong>{{ display.name }}</strong>
            <small>{{ display.summary }}</small>
          </div>
          <span :class="['display-status', display.status]">{{ display.status }}</span>
        </label>
      </div>

      <div class="viz-section">
        <div class="viz-section-header">
          <h3>Topic Preview</h3>
          <span class="viz-section-source">{{ topicData.source }}</span>
        </div>
        <div class="topic-preview-box" v-for="topic in topicData.topics" :key="topic.name">
          <div class="topic-title">
            <code>{{ topic.name }}</code>
            <span :class="['topic-status', topic.status]">{{ topic.status }}</span>
          </div>
          <span>{{ topic.dataType }} · {{ topic.summary }}</span>
          <small>{{ topic.source }} · {{ topic.messageRateHz }} Hz · {{ topic.lastSeen }}</small>
        </div>
      </div>

      <div class="viz-status-row">
        <span class="status-dot" :class="backendConnected ? 'on' : 'off'"></span>
        <span>{{ connectionLabel }}</span>
        <span class="viz-version">{{ topicData.message }}</span>
      </div>
    </aside>

    <article class="viz-panel viz-center">
      <div class="viz-panel-header">
        <h2>3D Viewport</h2>
        <span class="pill">Rerun SDK</span>
      </div>
      <div class="viewport-placeholder">
        <div class="viewport-grid-bg"></div>
        <div class="viewport-center">
          <div class="viewport-icon">R</div>
          <strong>Rerun Web Viewer</strong>
          <span class="viewport-hint">{{ viewportHint }}</span>
        </div>
      </div>
      <div class="viewport-controls">
        <button disabled>重置视角</button>
        <button disabled>俯视</button>
        <button disabled>跟随</button>
        <span class="fps-indicator">— fps</span>
      </div>
    </article>

    <aside class="viz-panel viz-right">
      <div class="viz-panel-header">
        <h2>Display Properties</h2>
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
  getDvizStatus,
  getDvizTopics,
  type ApiSource,
  type DvizDisplaysResponse,
  type DvizStatusResponse,
  type DvizTopicsResponse,
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

const dviz = ref<DvizStatusResponse>(fallbackDvizStatus)
const topicData = ref<DvizTopicsResponse>(fallbackTopicData)
const displayData = ref<DvizDisplaysResponse>(fallbackDisplayData)
const topicApiSource = ref<ApiSource>('fallback')
const statusApiSource = ref<ApiSource>('fallback')
const displayApiSource = ref<ApiSource>('fallback')

const backendConnected = computed(() => (
  statusApiSource.value === 'connected'
  || topicApiSource.value === 'connected'
  || displayApiSource.value === 'connected'
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

const viewportHint = computed(() => {
  const enabledDisplays = displayData.value.displays.filter((display) => display.enabled).length
  return `${enabledDisplays} displays · ${topicData.value.topics.length} topics · ${displayApiSource.value}`
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

onMounted(async () => {
  const [statusResult, topicResult, displayResult] = await Promise.all([
    getDvizStatus(fallbackDvizStatus),
    getDvizTopics(fallbackTopicData),
    getDvizDisplays(fallbackDisplayData),
  ])

  dviz.value = statusResult.data
  topicData.value = topicResult.data
  displayData.value = displayResult.data
  statusApiSource.value = statusResult.source
  topicApiSource.value = topicResult.source
  displayApiSource.value = displayResult.source
})
</script>
