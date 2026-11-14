<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { withBase } from 'vitepress'

interface Entry {
  id: string
  name: string
  category: string
  patterns: string[]
  description: string
  example: string
  since: string
  page: string
}
interface Category {
  id: string
  title: string
  page: string
}

const entries = ref<Entry[]>([])
const categories = ref<Category[]>([])
const version = ref('')
const error = ref('')
const query = ref('')
const active = ref('all')

onMounted(async () => {
  const hash = new URLSearchParams(location.hash.slice(1))
  query.value = hash.get('q') ?? ''
  active.value = hash.get('cat') ?? 'all'
  try {
    const res = await fetch(withBase('/syntax.json'))
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const data = await res.json()
    entries.value = data.entries
    categories.value = data.categories
    version.value = data.version
  } catch (e) {
    error.value = `Could not load syntax.json (${e}). Run npm run docs:gen.`
  }
})

// Keep the filter in the URL so a search can be shared.
watch([query, active], ([q, c]) => {
  const p = new URLSearchParams()
  if (q) p.set('q', q)
  if (c !== 'all') p.set('cat', c)
  const h = p.toString()
  history.replaceState(history.state, '', h ? `#${h}` : location.pathname)
})

const counts = computed(() => {
  const c: Record<string, number> = {}
  for (const e of entries.value) c[e.category] = (c[e.category] ?? 0) + 1
  return c
})

const filtered = computed(() => {
  const words = query.value.toLowerCase().split(/\s+/).filter(Boolean)
  return entries.value.filter((e) => {
    if (active.value !== 'all' && e.category !== active.value) return false
    if (!words.length) return true
    const hay = [e.name, e.description, e.example, ...e.patterns].join('\n').toLowerCase()
    return words.every((w) => hay.includes(w))
  })
})

function pageOf(e: Entry) {
  const cat = categories.value.find((c) => c.id === e.category)
  return withBase(`/${cat?.page ?? 'reference/'}#${e.id}`)
}

/** Escapes HTML, then turns `code` spans into <code>. */
function inline(text: string) {
  const esc = text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  return esc.replace(/`([^`]+)`/g, '<code>$1</code>')
}
</script>

<template>
  <div class="carve-search">
    <input
      v-model="query"
      class="carve-search-input"
      type="search"
      placeholder="Filter syntax: send, potion, %player%, list…"
      aria-label="Filter syntax"
    />
    <div class="carve-chips" role="group" aria-label="Category">
      <button :class="{ on: active === 'all' }" :aria-pressed="active === 'all'" @click="active = 'all'">
        All <span>{{ entries.length }}</span>
      </button>
      <button
        v-for="c in categories"
        :key="c.id"
        :class="{ on: active === c.id }"
        :aria-pressed="active === c.id"
        @click="active = c.id"
      >
        {{ c.title }} <span>{{ counts[c.id] ?? 0 }}</span>
      </button>
    </div>
    <p v-if="error" class="carve-error">{{ error }}</p>
    <p v-else class="carve-count">
      {{ filtered.length }} of {{ entries.length }} entries<span v-if="version"> · Carve {{ version }}</span>
    </p>
    <div class="carve-cards">
      <article v-for="e in filtered" :key="e.category + e.id" class="carve-card">
        <header>
          <a :href="pageOf(e)" class="carve-name">{{ e.name }}</a>
          <span class="carve-badge" :data-cat="e.category">{{ e.category }}</span>
        </header>
        <pre class="carve-patterns"><code>{{ e.patterns.join('\n') }}</code></pre>
        <p v-if="e.description" class="carve-desc" v-html="inline(e.description)"></p>
        <p v-else class="carve-desc carve-muted">No description yet.</p>
        <details v-if="e.example">
          <summary>Example</summary>
          <pre class="carve-example"><code>{{ e.example }}</code></pre>
        </details>
        <footer>
          <span>Since {{ e.since }}</span>
          <a v-if="e.page" :href="withBase(`/${e.page}`)">Guide →</a>
        </footer>
      </article>
    </div>
    <p v-if="!error && entries.length && !filtered.length" class="carve-count">
      Nothing matches. Try fewer words, or pick “All”.
    </p>
  </div>
</template>

<style scoped>
.carve-search {
  margin-top: 16px;
}
.carve-search-input {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-1);
  font-size: 15px;
}
.carve-search-input:focus {
  outline: 2px solid var(--vp-c-brand-2);
  outline-offset: 1px;
}
.carve-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 12px 0;
}
.carve-chips button {
  padding: 6px 12px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 6px;
  font-size: 13px;
  color: var(--vp-c-text-2);
  background: var(--vp-c-bg-alt);
}
.carve-chips button span {
  opacity: 0.7;
  margin-left: 4px;
}
.carve-chips button.on,
.carve-chips button:hover {
  border-color: var(--vp-c-brand-2);
  color: var(--vp-c-brand-1);
}
.carve-chips button:focus-visible {
  outline: 2px solid var(--vp-c-brand-2);
  outline-offset: 1px;
}
.carve-count,
.carve-error {
  font-size: 13px;
  color: var(--vp-c-text-3);
}
.carve-error {
  color: var(--vp-c-danger-1);
}
.carve-cards {
  display: grid;
  gap: 12px;
}
.carve-card {
  min-width: 0;
  padding: 12px 16px;
  border: 1px solid var(--vp-c-divider);
  border-left: 3px solid var(--vp-c-brand-3);
  border-radius: 8px;
  background: var(--vp-c-bg-soft);
}
.carve-card header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.carve-name {
  font-weight: 600;
  font-size: 16px;
  text-decoration: none;
}
.carve-badge {
  flex-shrink: 0;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1);
}
.carve-badge[data-cat='event'] { background: rgba(120, 200, 120, 0.15); color: #2f7a2f; }
.carve-badge[data-cat='condition'] { background: rgba(110, 160, 240, 0.15); color: #2c5fae; }
.carve-badge[data-cat='expression'] { background: rgba(200, 140, 240, 0.15); color: #7b3fae; }
.carve-badge[data-cat='type'] { background: rgba(240, 210, 110, 0.2); color: #7a6010; }
.carve-badge[data-cat='function'] { background: rgba(110, 220, 210, 0.15); color: #1d7a70; }
.dark .carve-badge[data-cat='event'] { color: #8fdc8f; }
.dark .carve-badge[data-cat='condition'] { color: #9dc0f5; }
.dark .carve-badge[data-cat='expression'] { color: #d3aaf5; }
.dark .carve-badge[data-cat='type'] { color: #ecd27a; }
.dark .carve-badge[data-cat='function'] { color: #7fe0d5; }
.carve-patterns,
.carve-example {
  margin: 10px 0;
  padding: 8px 12px;
  overflow-x: auto;
  border-radius: 6px;
  background: var(--vp-code-block-bg);
  font-size: 13px;
  line-height: 1.6;
  white-space: pre;
}
.carve-patterns code,
.carve-example code {
  padding: 0;
  background: none;
  color: var(--vp-c-text-1);
}
.carve-desc {
  margin: 6px 0;
  font-size: 14px;
  color: var(--vp-c-text-2);
}
.carve-muted {
  font-style: italic;
  color: var(--vp-c-text-3);
}
.carve-card details summary {
  cursor: pointer;
  font-size: 13px;
  color: var(--vp-c-text-2);
}
.carve-card footer {
  display: flex;
  justify-content: space-between;
  margin-top: 6px;
  font-size: 12px;
  color: var(--vp-c-text-3);
}
</style>
