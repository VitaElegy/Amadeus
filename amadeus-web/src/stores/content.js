import { defineStore } from 'pinia'

export const useContentStore = defineStore('content', {
  state: () => ({
    articles: [
      { id: 1, title: 'Amadeus Core Architecture', summary: 'Deep dive into Rust ownership model in message passing.', date: '2023-10-01', tag: 'Tech' },
      { id: 2, title: 'Spring Boot Integration Patterns', summary: 'Why JNA is better than REST for local IPC.', date: '2023-10-05', tag: 'DevOps' },
      { id: 3, title: 'Vue 3 Composition API Best Practices', summary: 'Stop writing spaghetti code in App.vue.', date: '2023-10-12', tag: 'Frontend' },
    ],
    todos: [
      { id: 1, content: 'Refactor Amadeus Message Bus', done: false },
      { id: 2, content: 'Update dependencies', done: true },
      { id: 3, content: 'Write decent documentation', done: false },
    ],
    memos: [
      { id: 1, content: 'Remember to check memory leaks in the unsafe Rust block.', time: '10:00 AM' },
      { id: 2, content: 'Meeting with the AI team at 2 PM.', time: '1:30 PM' },
    ],
    englishNotes: [
      { id: 1, word: 'Idempotent', definition: 'Denoting an element of a set which is unchanged in value when multiplied or otherwise operated on by itself.' },
      { id: 2, word: 'Ephemeral', definition: 'Lasting for a very short time.' },
    ]
  }),
  actions: {
    addTodo(content) {
      this.todos.push({ id: Date.now(), content, done: false });
    },
    toggleTodo(id) {
      const todo = this.todos.find(t => t.id === id);
      if (todo) todo.done = !todo.done;
    },
    deleteTodo(id) {
      this.todos = this.todos.filter(t => t.id !== id);
    },
    addMemo(content) {
        this.memos.unshift({ id: Date.now(), content, time: new Date().toLocaleTimeString() });
    }
  }
})

