import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'
import SyntaxSearch from './SyntaxSearch.vue'
import './custom.css'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('SyntaxSearch', SyntaxSearch)
  },
} satisfies Theme
