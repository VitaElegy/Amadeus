<script setup>
import { computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { 
  DashboardIcon, 
  ArticleIcon, 
  CheckCircleIcon, 
  NotebookIcon,
  LogoGithubIcon 
} from 'tdesign-icons-vue-next';

const route = useRoute();
const router = useRouter();

const activeValue = computed(() => route.path);

const changeHandler = (active) => {
  router.push(active);
};
</script>

<template>
  <t-layout class="h-screen w-screen bg-github-bg text-github-text overflow-hidden">
    <t-aside class="border-r border-github-border bg-github-card w-64 flex flex-col">
      <div class="p-6 border-b border-github-border flex items-center gap-3">
        <LogoGithubIcon size="32" class="text-white" />
        <span class="font-bold text-lg text-white tracking-tight">Amadeus</span>
      </div>
      
      <t-menu 
        theme="dark" 
        :value="activeValue" 
        class="bg-transparent flex-1 mt-4"
        @change="changeHandler"
      >
        <t-menu-item value="/" class="text-gray-300 hover:text-white">
          <template #icon><DashboardIcon /></template>
          Dashboard
        </t-menu-item>
        <t-menu-item value="/articles" class="text-gray-300 hover:text-white">
          <template #icon><ArticleIcon /></template>
          Knowledge Base
        </t-menu-item>
        <t-menu-item value="/todos" class="text-gray-300 hover:text-white">
          <template #icon><CheckCircleIcon /></template>
          Task Master
        </t-menu-item>
        <t-menu-item value="/notes" class="text-gray-300 hover:text-white">
          <template #icon><NotebookIcon /></template>
          Memos & Notes
        </t-menu-item>
      </t-menu>
      
      <div class="p-4 border-t border-github-border text-xs text-gray-500 text-center">
        v2.0.0 Enterprise Edition
      </div>
    </t-aside>

    <t-layout>
      <t-content class="p-8 overflow-y-auto bg-github-bg relative">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </t-content>
    </t-layout>
  </t-layout>
</template>

<style scoped>
:deep(.t-menu__item) {
  transition: all 0.2s ease;
}
:deep(.t-menu__item.t-is-active) {
  background-color: #1f6feb22 !important;
  color: #58a6ff !important;
  border-right: 3px solid #58a6ff;
}
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

