<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'

type DependencyStatus = {
  name: string
  available: boolean
  version?: string
  path?: string
  required: boolean
}

type FormatSummary = {
  height?: number
  ext?: string
  fps?: number
  videoCodec?: string
  audioCodec?: string
}

type VideoInfo = {
  resolvedUrl: string
  title: string
  duration?: number
  thumbnail?: string
  extractor?: string
  uploader?: string
  width?: number
  height?: number
  formats: FormatSummary[]
}

type DownloadEvent = {
  taskId: string
  state: 'downloading' | 'processing' | 'completed' | 'failed' | 'cancelled'
  percent?: number
  speed?: string
  eta?: string
  filepath?: string
  message?: string
}

const url = ref('')
const outputDir = ref('')
const profile = ref('best')
const info = ref<VideoInfo | null>(null)
const dependencies = ref<DependencyStatus[]>([])
const busy = ref(false)
const parsing = ref(false)
const errorMessage = ref('')
const taskId = ref('')
const lastEvent = ref<DownloadEvent | null>(null)
const runtimeReady = ref(true)
let unlisten: UnlistenFn | undefined

const requiredDependencies = computed(() => dependencies.value.filter((item) => item.required))
const missingDependencies = computed(() => requiredDependencies.value.filter((item) => !item.available))
const canDownload = computed(
  () => Boolean(info.value) && dependencies.value.length > 0 && !busy.value && missingDependencies.value.length === 0,
)
const progress = computed(() => Math.max(0, Math.min(100, lastEvent.value?.percent ?? 0)))
const taskLabel = computed(() => {
  switch (lastEvent.value?.state) {
    case 'downloading':
      return '正在提取视频'
    case 'processing':
      return '正在合并视频'
    case 'completed':
      return '下载完成'
    case 'failed':
      return '下载失败'
    case 'cancelled':
      return '已取消'
    default:
      return '等待下载'
  }
})

function formatDuration(seconds?: number) {
  if (!seconds || seconds < 1) return '时长未知'
  const total = Math.round(seconds)
  const hours = Math.floor(total / 3600)
  const minutes = Math.floor((total % 3600) / 60)
  const remaining = total % 60
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, '0')}:${String(remaining).padStart(2, '0')}`
    : `${minutes}:${String(remaining).padStart(2, '0')}`
}

function clearError() {
  errorMessage.value = ''
}

function handleUrlInput() {
  info.value = null
  lastEvent.value = null
  clearError()
}

async function parseUrl() {
  clearError()
  info.value = null
  lastEvent.value = null
  if (!url.value.trim()) {
    errorMessage.value = '先粘贴一个视频链接。'
    return
  }
  if (!isTauri()) {
    errorMessage.value = '请通过桌面模式运行：npm run tauri:dev。'
    return
  }
  parsing.value = true
  try {
    info.value = await invoke<VideoInfo>('inspect_url', { url: url.value.trim() })
  } catch (error) {
    errorMessage.value = String(error)
  } finally {
    parsing.value = false
  }
}

async function chooseOutputDir() {
  clearError()
  if (!isTauri()) return
  const selected = await open({ directory: true, multiple: false, title: '选择视频保存目录' })
  if (typeof selected === 'string') outputDir.value = selected
}

async function startDownload() {
  clearError()
  if (!info.value) {
    await parseUrl()
    if (!info.value) return
  }
  if (!outputDir.value) {
    errorMessage.value = '请选择保存目录。'
    return
  }
  if (!isTauri()) return
  busy.value = true
  lastEvent.value = null
  try {
    taskId.value = await invoke<string>('start_download', {
      request: {
        url: info.value.resolvedUrl || url.value.trim(),
        outputDir: outputDir.value,
        profile: profile.value,
      },
    })
  } catch (error) {
    busy.value = false
    errorMessage.value = String(error)
  }
}

async function cancelDownload() {
  if (!taskId.value) return
  try {
    await invoke('cancel_download', { taskId: taskId.value })
  } catch (error) {
    errorMessage.value = String(error)
  }
}

async function openPath(path?: string) {
  if (!path || !isTauri()) return
  try {
    await invoke('open_path', { path })
  } catch (error) {
    errorMessage.value = String(error)
  }
}

async function loadRuntimeState() {
  if (!isTauri()) {
    runtimeReady.value = false
    outputDir.value = '请运行桌面模式后选择目录'
    return
  }
  try {
    outputDir.value = await invoke<string>('default_output_dir')
    dependencies.value = await invoke<DependencyStatus[]>('check_dependencies')
  } catch (error) {
    errorMessage.value = String(error)
  }
}

onMounted(async () => {
  if (isTauri()) {
    unlisten = await listen<DownloadEvent>('download://progress', (event) => {
      lastEvent.value = event.payload
      if (['completed', 'failed', 'cancelled'].includes(event.payload.state)) busy.value = false
    })
  }
  await loadRuntimeState()
})

onUnmounted(() => unlisten?.())
</script>

<template>
  <main class="app-shell">
    <header class="topbar">
      <div class="brand-lockup">
        <div class="brand-mark">VL</div>
        <div>
          <p class="eyebrow">LOCAL MEDIA TOOL</p>
          <h1>视频链接提取器</h1>
        </div>
      </div>
      <div class="local-badge"><span class="status-dot"></span>只在本机运行</div>
    </header>

    <section class="hero-grid">
      <div class="hero-copy">
        <p class="eyebrow accent">PASTE · RESOLVE · SAVE</p>
        <h2>把网页里的视频，<em>干净地</em>保存下来。</h2>
        <p class="hero-text">输入一个视频链接，应用会在本机解析画质并下载到你指定的文件夹。不会启动公网服务，也不会把链接交给第三方网站。</p>
      </div>
      <div class="hero-note">
        <span class="note-number">01</span>
        <p>远程视频仍需要访问原站。解析、队列、日志和文件都留在这台电脑上。</p>
      </div>
    </section>

    <section class="workspace-card">
      <div class="section-heading">
        <div>
          <p class="eyebrow">NEW EXTRACTION</p>
          <h3>添加一个视频链接</h3>
        </div>
        <span class="step-chip">STEP 01 / 03</span>
      </div>

      <label class="field-label" for="url">视频链接或分享文案</label>
      <div class="url-row">
        <input id="url" v-model="url" type="text" placeholder="粘贴视频链接或平台分享文字" autocomplete="off" @input="handleUrlInput" @keydown.enter="parseUrl" />
        <button class="primary-button" :disabled="parsing || !url.trim()" @click="parseUrl">
          {{ parsing ? '解析中…' : '解析链接' }}
          <span aria-hidden="true">↗</span>
        </button>
      </div>
      <p class="field-hint">支持直接粘贴带说明文字的分享内容，解析时会自动提取其中的链接。</p>

      <div v-if="errorMessage" class="error-box" role="alert">{{ errorMessage }}</div>

      <div v-if="info" class="video-preview">
        <div class="preview-cover">
          <img v-if="info.thumbnail" :src="info.thumbnail" alt="视频封面" />
          <div v-else class="cover-fallback">VIDEO</div>
          <span class="duration-pill">{{ formatDuration(info.duration) }}</span>
        </div>
        <div class="preview-details">
          <p class="eyebrow accent">{{ info.extractor || 'VIDEO SOURCE' }}</p>
          <h4>{{ info.title }}</h4>
          <p class="muted">{{ info.uploader || '未知作者' }} · {{ info.width && info.height ? `${info.width} × ${info.height}` : '可用画质待选择' }}</p>
          <div class="format-pills">
            <span v-for="format in info.formats.slice(0, 5)" :key="`${format.height}-${format.ext}-${format.fps}`">
              {{ format.height ? `${format.height}p` : '自适应' }} {{ format.ext || '' }}
            </span>
          </div>
        </div>
      </div>

      <div class="options-grid">
        <div>
          <label class="field-label" for="profile">画质策略</label>
          <select id="profile" v-model="profile">
            <option value="best">最佳可用 · 保留原画质</option>
            <option value="1080">1080p 优先</option>
            <option value="720">720p 优先</option>
          </select>
        </div>
        <div>
          <label class="field-label" for="output">保存到</label>
          <div class="path-row">
            <input id="output" v-model="outputDir" type="text" :disabled="!runtimeReady" />
            <button class="secondary-button" :disabled="!runtimeReady" @click="chooseOutputDir">选择</button>
          </div>
        </div>
      </div>

      <div class="action-row">
        <div class="action-hint"><span class="hint-icon">⌁</span>默认只处理当前视频，不展开播放列表</div>
        <div class="button-group">
          <button v-if="busy" class="secondary-button danger" @click="cancelDownload">取消下载</button>
          <button class="download-button" :disabled="!canDownload" @click="startDownload">
            {{ busy ? '下载中…' : '开始下载' }}
            <span aria-hidden="true">↓</span>
          </button>
        </div>
      </div>
    </section>

    <section class="lower-grid">
      <div class="task-card">
        <div class="section-heading compact">
          <div>
            <p class="eyebrow">CURRENT TASK</p>
            <h3>{{ taskLabel }}</h3>
          </div>
          <span v-if="lastEvent" class="task-state" :class="lastEvent.state">{{ lastEvent.state }}</span>
        </div>
        <div class="progress-track"><span :style="{ width: `${progress}%` }"></span></div>
        <div class="task-meta">
          <span>{{ lastEvent?.speed || '等待速度' }}</span>
          <span>{{ lastEvent?.eta ? `剩余 ${lastEvent.eta}` : `${Math.round(progress)}%` }}</span>
        </div>
        <div v-if="lastEvent?.filepath" class="completed-row">
          <span class="success-mark">✓</span>
          <span class="filepath">{{ lastEvent.filepath }}</span>
          <button class="text-button" @click="openPath(lastEvent.filepath)">打开文件</button>
        </div>
      </div>

      <div class="runtime-card">
        <div class="section-heading compact">
          <div>
            <p class="eyebrow">LOCAL RUNTIME</p>
            <h3>运行时状态</h3>
          </div>
          <span class="runtime-lock">LOCKED</span>
        </div>
        <div v-if="dependencies.length" class="dependency-list">
          <div v-for="dependency in dependencies" :key="dependency.name" class="dependency-row">
            <span :class="['dependency-dot', { ready: dependency.available }]" aria-hidden="true"></span>
            <span>{{ dependency.name }}</span>
            <small>{{ dependency.available ? dependency.version : dependency.required ? '缺少' : '可选' }}</small>
          </div>
        </div>
        <p v-else class="muted runtime-hint">桌面模式启动后会检查 yt-dlp、FFmpeg 和 Deno。</p>
      </div>
    </section>

    <footer class="footer-note"><span>yt-dlp + FFmpeg</span><span>·</span><span>本机处理，不提供公网访问</span></footer>
  </main>
</template>
