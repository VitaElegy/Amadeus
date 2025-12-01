<script setup>
import { ref } from 'vue';
import { useContentStore } from '../stores/content';
import { AddIcon, DeleteIcon, CheckIcon } from 'tdesign-icons-vue-next';

const store = useContentStore();
const newTodo = ref('');

const add = () => {
    if(!newTodo.value) return;
    store.addTodo(newTodo.value);
    newTodo.value = '';
}
</script>

<template>
  <div class="max-w-3xl mx-auto space-y-8">
    <header class="text-center">
       <h1 class="text-3xl font-bold text-white mb-2">Task Master</h1>
       <p class="text-gray-500">Focus on what matters.</p>
    </header>

    <div class="bg-github-card border border-github-border rounded-lg p-6 shadow-lg">
        <div class="flex gap-2 mb-6">
            <t-input v-model="newTodo" placeholder="What needs to be done?" @enter="add" size="large" />
            <t-button theme="primary" @click="add" shape="square" size="large">
                <template #icon><AddIcon /></template>
            </t-button>
        </div>

        <t-list :split="true" class="bg-transparent">
            <t-list-item v-for="todo in store.todos" :key="todo.id" class="hover:bg-github-bg transition-colors rounded px-2">
                <div class="flex items-center gap-3 w-full">
                    <div 
                        class="w-5 h-5 rounded-full border border-gray-500 flex items-center justify-center cursor-pointer hover:border-green-500"
                        :class="{'bg-green-600 border-green-600': todo.done}"
                        @click="store.toggleTodo(todo.id)"
                    >
                        <CheckIcon v-if="todo.done" size="14" class="text-white" />
                    </div>
                    <span 
                        class="flex-1 text-base transition-all"
                        :class="{'line-through text-gray-600': todo.done, 'text-gray-200': !todo.done}"
                    >
                        {{ todo.content }}
                    </span>
                    <t-button 
                        theme="danger" variant="text" shape="circle" size="small"
                        @click="store.deleteTodo(todo.id)"
                    >
                        <template #icon><DeleteIcon /></template>
                    </t-button>
                </div>
            </t-list-item>
        </t-list>
    </div>
  </div>
</template>

<style scoped>
:deep(.t-list-item) {
    padding: 12px 8px;
}
</style>

