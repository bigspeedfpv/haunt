import { createPinia } from "pinia";
import { createApp } from "vue";
import { createMemoryHistory, createRouter } from "vue-router";
import devtools from "@vue/devtools";

import App from "@/App.vue";
import LoginView from "@/routes/LoginView.vue";
import PregameView from "@/routes/PregameView.vue";
import IngameView from "@/routes/IngameView.vue";

import "@/styles.css";

const routes = [
  { path: "/", component: LoginView },
  { path: "/pregame", component: PregameView },
  { path: "/ingame", component: IngameView },
];

const router = createRouter({
  history: createMemoryHistory(),
  routes,
});

const pinia = createPinia();

if (process.env.NODE_ENV === "development") {
  devtools.connect("http://localhost", 8098);
}

createApp(App).use(router).use(pinia).mount("#app");
