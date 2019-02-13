import Vue from 'vue'
import VueRouter from 'vue-router';
import Vuex from 'vuex';

import { MdButton, MdContent, MdTabs, MdApp, MdIcon, MdToolbar, MdDrawer, MdList } from 'vue-material/dist/components'
import 'vue-material/dist/vue-material.min.css'
import 'vue-material/dist/theme/default.css'

import Home from './components/Home.vue';
import Record from './components/Record';

Vue.use(MdButton);
Vue.use(MdContent);
Vue.use(MdTabs);
Vue.use(MdApp);
Vue.use(MdIcon);
Vue.use(MdToolbar);
Vue.use(MdDrawer);
Vue.use(MdList);
Vue.use(VueRouter);
Vue.use(Vuex);

import App from './App.vue'

Vue.config.productionTip = false;

const store = new Vuex.Store({
  state: {
    recordId: String,
    recordContent: {
      ttl: Number,
      content: String
    }
  },
  mutations: {
    setRecordId(state, payload) {
      state.recordId = payload;
    }
  },
  actions: {
    changeRecordId({commit}, id) {
      commit('setRecordId', id);
    }
  }
});

const routes = [
  { path: '/', component: Home },
  { path: '/record/:id', component: Record, props: true }
];

const router = new VueRouter({
  routes
});

new Vue({
  render: h => h(App),
  store,
  router: router
}).$mount('#app');
