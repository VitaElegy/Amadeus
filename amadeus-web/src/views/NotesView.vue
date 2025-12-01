<script setup>
import { ref } from 'vue';
import { useContentStore } from '../stores/content';
import { ChatIcon, BookIcon } from 'tdesign-icons-vue-next';

const store = useContentStore();
const memoInput = ref('');

const addMemo = () => {
    if(!memoInput.value) return;
    store.addMemo(memoInput.value);
    memoInput.value = '';
}
</script>

<template>
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
    
    <!-- Memos Section -->
    <div class="space-y-4">
        <h2 class="text-xl font-bold text-white flex items-center gap-2">
            <ChatIcon /> Quick Memos
        </h2>
        
        <div class="bg-github-card border border-github-border rounded-lg p-4">
            <t-textarea 
                v-model="memoInput" 
                placeholder="Type a quick note..." 
                :autosize="{ minRows: 2, maxRows: 5 }"
                class="mb-2"
            />
            <div class="flex justify-end">
                <t-button theme="primary" size="small" @click="addMemo">Post Memo</t-button>
            </div>
        </div>

        <div class="space-y-4">
            <div 
                v-for="memo in store.memos" :key="memo.id"
                class="bg-github-card border border-github-border rounded p-4 relative pl-8"
            >
                <div class="absolute left-0 top-0 bottom-0 w-1 bg-yellow-600 rounded-l"></div>
                <p class="text-gray-300 whitespace-pre-wrap">{{ memo.content }}</p>
                <span class="text-xs text-gray-500 mt-2 block">{{ memo.time }}</span>
            </div>
        </div>
    </div>

    <!-- English Notes Section -->
    <div class="space-y-4">
        <h2 class="text-xl font-bold text-white flex items-center gap-2">
            <BookIcon /> Vocabulary
        </h2>

        <div class="grid grid-cols-1 gap-4">
            <div 
                v-for="note in store.englishNotes" :key="note.id"
                class="bg-github-card border border-github-border rounded p-4 group hover:border-blue-500 transition-colors"
            >
                <div class="flex justify-between items-baseline mb-2">
                    <h3 class="text-lg font-bold text-blue-400 font-serif">{{ note.word }}</h3>
                    <span class="text-xs px-2 py-0.5 rounded bg-gray-700 text-gray-300">noun</span>
                </div>
                <p class="text-gray-400 italic font-serif">"{{ note.definition }}"</p>
            </div>
        </div>
    </div>

  </div>
</template>

