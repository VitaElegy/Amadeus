<script setup>
import { useContentStore } from '../stores/content';
import { CalendarIcon, TagIcon } from 'tdesign-icons-vue-next';

const store = useContentStore();
</script>

<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center">
        <h1 class="text-2xl font-bold text-white">Knowledge Base</h1>
        <t-button theme="default" variant="outline">New Article</t-button>
    </div>

    <div class="grid gap-4">
      <t-card 
        v-for="article in store.articles" 
        :key="article.id" 
        :bordered="false"
        class="bg-github-card border border-github-border hover:border-gray-500 transition-colors cursor-pointer group"
      >
        <div class="flex justify-between items-start">
          <div>
            <h3 class="text-xl font-semibold text-blue-400 group-hover:underline mb-2">
              {{ article.title }}
            </h3>
            <p class="text-gray-400 mb-4">{{ article.summary }}</p>
            <div class="flex gap-4 text-xs text-gray-500">
              <span class="flex items-center gap-1"><CalendarIcon /> {{ article.date }}</span>
              <span class="flex items-center gap-1"><TagIcon /> {{ article.tag }}</span>
            </div>
          </div>
        </div>
      </t-card>
    </div>
  </div>
</template>

<style scoped>
:deep(.t-card) {
    background-color: #161b22;
    color: #c9d1d9;
}
</style>

