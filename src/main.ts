import { createPinia } from "pinia";
import { createApp } from "vue";
import { createMemoryHistory, createRouter } from "vue-router";

import App from "@/App.vue";
import LoginView from "@/routes/LoginView.vue";
import PregameView from "@/routes/PregameView.vue";

import "@/styles.css";

const routes = [
  { path: "/", component: LoginView },
  { path: "/pregame", component: PregameView },
];

const router = createRouter({
  history: createMemoryHistory(),
  routes,
});

const pinia = createPinia();

createApp(App).use(router).use(pinia).mount("#app");
