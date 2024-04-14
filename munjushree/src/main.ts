import 'vant/lib/index.css';
import 'leaflet/dist/leaflet.css';
import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'

import { Toast, FloatingPanel, Form, Field, CellGroup, Button, Overlay, Notify } from 'vant';

import App from './App.vue'
import router from './router'

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.use(Toast).use(FloatingPanel).use(Form).use(Field).use(CellGroup).use(Button).use(Overlay).use(Notify)

app.mount('#app')

document.addEventListener("contextmenu", (e) => {e.preventDefault()});