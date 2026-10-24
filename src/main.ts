import { createApp } from 'vue'
import App from './App.vue'
import './assets/index.css'
import { applyTheme, initialDark } from './theme'

applyTheme(initialDark(), false)
createApp(App).mount('#app')
